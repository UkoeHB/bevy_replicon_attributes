//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::{Component, Deref};
use siphasher::sip128::{Hasher128, SipHasher13};
use smallvec::SmallVec;

//standard shortcuts
use std::any::TypeId;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// A node within a visibility condition tree.
/// - The root node records a visibility attribute.
/// - Non-root nodes record indices into the flattened condition tree corresponding to their children.
#[derive(Debug, Copy, Clone, Default, Hash)]
pub enum VisibilityConditionNode
{
    #[default]
    Empty,
    /// Root node.
    Attr(VisibilityAttributeId),
    Not(usize),
    And(usize, usize),
    Or(usize, usize),
}

//-------------------------------------------------------------------------------------------------------------------

/// Evaluates a condition branch with the given root node inspector.
fn evaluate(
    inspector    : &impl Fn(VisibilityAttributeId) -> bool,
    condition    : &[VisibilityConditionNode],
    current_node : usize
) -> bool
{
    match condition[current_node]
    {
        VisibilityConditionNode::Empty =>
        { tracing::error!("encountered empty node when evaluating condition tree"); false }
        VisibilityConditionNode::Attr(attr) => (inspector)(attr),
        VisibilityConditionNode::Not(a)     => !evaluate(inspector, condition, a),
        VisibilityConditionNode::And(a, b)  => evaluate(inspector, condition, a) && evaluate(inspector, condition, b),
        VisibilityConditionNode::Or(a, b)   => evaluate(inspector, condition, a) || evaluate(inspector, condition, b),
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Id associated with a visibility attribute.
///
/// This is used to differentiate visibility attributes within the attribute engine's internal maps.
/// Equivalent attribute instances should have equal attribute ids, and non-equivalent attribute instances should
/// have unequal attribute ids.
///
/// Since the inner id is 64 bits, this id can be considered to have 64 bits of collision resistance.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct VisibilityAttributeId(TypeId, u64);

impl VisibilityAttributeId
{
    /// Makes a new visibility attribute id.
    pub(crate) fn new<T: 'static>(inner_id: u64) -> Self
    {
        Self(TypeId::of::<T>(), inner_id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Id associated with a visibility condition.
///
/// This is used to differentiate visibility conditions within the attribute engine's internal maps.
/// The id has 64 bits of collision resistance, which should be adequate for the vast majority of use-cases.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct VisibilityConditionId(u128);

//-------------------------------------------------------------------------------------------------------------------

/// Signifies that a type is a visibility attribute.
///
/// The `VisibilityAttribute` derive macro will derive [`DefaultVisibilityAttribute`] on your type.
///
/**
**Examples**

With a derive:
```rust
#[derive(VisibilityAttribute, Default, Eq, PartialEq)]
struct InCastle;
```

Manually implemented:
```rust
struct InCastleRoom(u16);

impl VisibilityAttribute for InCastleRoom
{
    fn inner_attribute_id(&self) -> u64 { self.0 as u64 }
}
```
*/
pub trait VisibilityAttribute: Sized + 'static
{
    /// Returns the inner id of this attribute.
    ///
    /// If your attribute contains non-type information (e.g. a client id), then you should manually implement this
    /// trait.
    ///
    /// See [`attribute_id`] for how to get a [`VisibilityAttributeId`].
    /// Note that ids are domain-separated by attribute type, so you can safely use the full `u64` range to define your
    /// inner id.
    fn inner_attribute_id(&self) -> u64;

    /// Returns the id of an attribute.
    ///
    /// By default the id is a concatenation of the attribute's type id and its inner id.
    fn attribute_id(&self) -> VisibilityAttributeId
    {
        VisibilityAttributeId::new::<Self>(self.inner_attribute_id())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Implemented by the derive for [`VisibilityAttribute`].
///
/// This trait requires `Default`, `Eq`, and `PartialEq` to enforce that only default-constructed objects can be
/// assigned a default visibility attribute id.
///
/// [`VisibilityAttribute::inner_attribute_id`] will panic on types that implement this if the attribute does
/// not equal its default value.
/// If that happens, you should manually implement [`VisibilityAttribute`] and define an appropriate inner attribute id
/// for your type.
pub trait DefaultVisibilityAttribute: Default + Eq + PartialEq + 'static {}

impl<T: DefaultVisibilityAttribute> VisibilityAttribute for T
{
    fn inner_attribute_id(&self) -> u64
    {
        if *self != Self::default()
        { panic!("non-default-constructed objects should implement VisibilityAttribute manually"); }

        0u64
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents a type that can convert into a visibility condition.
///
/// The type that implements this can't be used direct. You need to extract it with [`VisibilityCondition::new`].
/// Then the condition can be evaluated with [`VisibilityCondition::evaluate`].
pub trait IntoVisibilityCondition: 'static
{
    /// Builds the condition expression.
    fn build(self, builder: VisibilityConditionBuilder) -> VisibilityConditionBuilder;
}

//-------------------------------------------------------------------------------------------------------------------

pub enum VisibilityConditionWrapper<E>
where
    E: VisibilityConditionExpression,
{
    Root(VisibilityAttributeId),
    Expression(E),
}

impl<A> From<A> for VisibilityConditionWrapper<DummyVisClosure>
where
    A: VisibilityAttribute,
{
    fn from(a: A) -> Self
    {
        Self::Root(a.attribute_id())
    }
}

impl<E> From<E> for VisibilityConditionWrapper<E>
where
    E: VisibilityConditionExpression,
{
    fn from(e: E) -> Self
    {
        Self::Expression(e)
    }
}

impl<E> IntoVisibilityCondition for VisibilityConditionWrapper<E>
where
    E: VisibilityConditionExpression,
{
    fn build(self, mut builder: VisibilityConditionBuilder) -> VisibilityConditionBuilder
    {
        match self
        {
            Self::Root(id) =>
            {
                builder.push_attr_node(id);
                builder
            }
            Self::Expression(expr) =>
            {
                (expr)(builder)
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Creates an ATTRIBUTE visibility condition.
pub fn attr<A>(a: A) -> impl IntoVisibilityCondition
where
    A: VisibilityAttribute + 'static,
{
    VisibilityConditionWrapper::from(a)
}

/// Creates a NOT visibility condition.
pub fn not<C>(a: C) -> impl IntoVisibilityCondition
where
    C: IntoVisibilityCondition + 'static,
{
    VisibilityConditionWrapper::from(
        move |mut builder: VisibilityConditionBuilder| -> VisibilityConditionBuilder
        {
            builder.push_not_node();
            a.build(builder)
        }
    )
}

/// Creates an AND visibility condition.
pub fn and<A, B>(a: A, b: B) -> impl IntoVisibilityCondition
where
    A: IntoVisibilityCondition + 'static,
    B: IntoVisibilityCondition + 'static
{
    VisibilityConditionWrapper::from(
        move |mut builder: VisibilityConditionBuilder| -> VisibilityConditionBuilder
        {
            let and_node = builder.push_empty(2);
            let mut builder = a.build(builder);
            builder.set_and_node(and_node);
            b.build(builder)
        }
    )
}

/// Creates an OR visibility condition.
pub fn or<A, B>(a: A, b: B) -> impl IntoVisibilityCondition
where
    A: IntoVisibilityCondition + 'static,
    B: IntoVisibilityCondition + 'static
{
    VisibilityConditionWrapper::from(
        move |mut builder: VisibilityConditionBuilder| -> VisibilityConditionBuilder
        {
            let or_node = builder.push_empty(2);
            let mut builder = a.build(builder);
            builder.set_or_node(or_node);
            b.build(builder)
        }
    )
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents a visibility condition expression builder.
pub trait VisibilityConditionExpression: FnOnce(VisibilityConditionBuilder) -> VisibilityConditionBuilder + 'static
{}

impl<F> VisibilityConditionExpression for F
where
    F: FnOnce(VisibilityConditionBuilder) -> VisibilityConditionBuilder + 'static
{}

pub type DummyVisClosure = fn(VisibilityConditionBuilder) -> VisibilityConditionBuilder;

//-------------------------------------------------------------------------------------------------------------------

/// Visibility condition builder.
pub struct VisibilityConditionBuilder
{
    nodes: SmallVec<[VisibilityConditionNode; SMALL_PACK_LEN]>,
}

impl VisibilityConditionBuilder
{
    /// Creates a new condition builder.
    fn new() -> Self
    {
        Self{ nodes: SmallVec::default() }
    }

    /// Pushes an empty node which will be set later.
    ///
    /// Allows defining how many `extra` nodes are associated with this node, to improve reallocation accuracy.
    fn push_empty(&mut self, extra: usize) -> usize
    {
        let position = self.nodes.len();
        self.nodes.reserve(extra + 1);
        self.nodes.push(VisibilityConditionNode::Empty);
        position
    }

    /// Adds an ATTRIBUTE node to the end of the condition.
    fn push_attr_node(&mut self, attr: VisibilityAttributeId)
    {
        self.nodes.push(VisibilityConditionNode::Attr(attr));
    }

    /// Adds a NOT node to the end of the condition.
    ///
    /// Assumes the next node to be inserted will be the start of the OR expression's child branch.
    fn push_not_node(&mut self)
    {
        let next_node = self.nodes.len() + 1;
        self.nodes.reserve(2);
        self.nodes.push(VisibilityConditionNode::Not(next_node));
    }

    /// Sets an AND node at its branch root position.
    ///
    /// Assumes the next node to be inserted will be the start of the AND expression's right branch.
    ///
    /// Panics if the AND node position was not inserted with [`Self::push_empty`].
    fn set_and_node(&mut self, node: usize)
    {
        let left = node + 1;
        let right = self.nodes.len();
        self.nodes[node] = VisibilityConditionNode::And(left, right);
    }

    /// Sets an OR node at its branch root position.
    ///
    /// Assumes the next node to be inserted will be the start of the OR expression's right branch.
    ///
    /// Panics if the OR node position was not inserted with [`Self::push_empty`].
    fn set_or_node(&mut self, node: usize)
    {
        let left = node + 1;
        let right = self.nodes.len();
        self.nodes[node] = VisibilityConditionNode::Or(left, right);
    }

    /// Takes the internal nodes.
    fn take(self) -> SmallVec<[VisibilityConditionNode; SMALL_PACK_LEN]>
    {
        self.nodes
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Max number of nodes for non-allocating [`VisibilityCondition`]s.
pub const SMALL_PACK_LEN: usize = 3;

/// A type-erased visibility condition.
///
/// Constructing a condition only requires allocations if the condition contains more than [`SMALL_PACK_LEN`] nodes.
///
/// Cloning a condition will *not* allocate.
///
/// Examples:
/// - 1 node: `VisibleTo::new(attr(Global))`
/// - 2 nodes: `VisibleTo::new(not(attr(InABush)))`
/// - 3 nodes: `VisibleTo::new(and(attr(IsFast), attr(IsSmall))`
/// - 4 nodes: `VisibleTo::new(and(attr(IsSwimming), not(attr(WearingSwimsuit))))`
#[derive(Debug, Clone, Hash)]
pub enum VisibilityCondition
{
    Small(SmallVec<[VisibilityConditionNode; SMALL_PACK_LEN]>),
    Large(Arc<[VisibilityConditionNode]>),
}

impl VisibilityCondition
{
    /// Makes a new condition with the given node tree.
    pub fn new(condition: impl IntoVisibilityCondition) -> Self
    {
        let builder = VisibilityConditionBuilder::new();
        let final_builder = condition.build(builder);
        let condition = final_builder.take();

        if !condition.spilled()
        {
            Self::Small(condition)
        }
        else
        {
            Self::Large(Arc::from(condition.into_vec()))
        }
    }

    /// Gets the condition id.
    ///
    /// Note that this requires hashing the internal condition, which may be expensive.
    /// We don't cache the id here since it is 16 bytes.
    pub fn condition_id(&self) -> VisibilityConditionId
    {
        let mut hasher = SipHasher13::new();
        self.hash(&mut hasher);
        let id = hasher.finish128();

        VisibilityConditionId(id.into())
    }

    /// Iterates attributes within the condition tree.
    pub fn iter_attributes(&self) -> impl Iterator<Item = VisibilityAttributeId> + '_
    {
        let filter = |n: &VisibilityConditionNode| -> Option<VisibilityAttributeId>
        {
            let VisibilityConditionNode::Attr(attr) = n else { return None; };
            Some(*attr)
        };
        match self
        {
            Self::Small(condition) => condition.iter().filter_map(filter),
            Self::Large(condition) => condition.iter().filter_map(filter),
        }
    }

    /// Evaluates the condition tree with the attribute evaluator.
    ///
    /// Modifiers are automatically evaluated. The evaluator only checks if the given attribute is known.
    pub fn evaluate(&self, evaluator: impl Fn(VisibilityAttributeId) -> bool) -> bool
    {
        match self
        {
            Self::Small(condition) => evaluate(&evaluator, condition.as_slice(), 0),
            Self::Large(condition) => evaluate(&evaluator, condition, 0),
        }
    }
}

impl PartialEq for VisibilityCondition
{
    fn eq(&self, other: &Self) -> bool
    {
        self.condition_id() == other.condition_id()
    }
}
impl Eq for VisibilityCondition {}

//-------------------------------------------------------------------------------------------------------------------

/// Component that records the visibility for an entity.
///
/// Derefs to a [`VisibilityCondition`].
#[derive(Component, Debug, Clone, Deref)]
pub struct VisibleTo(VisibilityCondition);

impl VisibleTo
{
    /// Makes a new `VisibleTo` component.
    pub fn new(condition: impl IntoVisibilityCondition + 'static) -> Self
    {
        Self(VisibilityCondition::new(condition))
    }
}

impl PartialEq for VisibleTo
{
    fn eq(&self, other: &Self) -> bool
    {
        self.0 == other.0
    }
}
impl Eq for VisibleTo {}

//-------------------------------------------------------------------------------------------------------------------
