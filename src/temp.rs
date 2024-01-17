//local shortcuts
use crate::*;

//third-party shortcuts
use siphasher::sip128::{Hasher128, SipHasher13};

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
pub(crate) enum VisibilityConditionNode
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
        VisibilityConditionNode::Empty      => { tracing::error!("encountered empty node when evaluating condition tree"); false },
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

/// Returns the id of an attribute.
///
/// The id is a concatenation of the attribute's type id and its inner id.
pub fn attribute_id<T: VisibilityAttribute + 'static>(attribute: &T) -> VisibilityAttributeId
{
    VisibilityAttributeId::new::<T>(attribute.inner_attribute_id())
}

//-------------------------------------------------------------------------------------------------------------------

/// Id associated with a visibility condition.
///
/// This is used to differentiate visibility conditions within the attribute engine's internal maps.
/// The id has 64 bits of collision resistance, which should be adequate for the vast majority of use-cases.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub(crate) struct VisibilityConditionId(u128);

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
pub trait VisibilityAttribute
{

    /// Returns the inner id of this attribute.
    ///
    /// If your attribute contains non-type information (e.g. a client id), then you should manually implement this
    /// trait.
    ///
    /// See [`attribute_id`] for how to get a [`VisibilityAttributeId`].
    fn inner_attribute_id(&self) -> u64;
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
pub trait DefaultVisibilityAttribute: Default + Eq + PartialEq {}

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

/// Represents a visibility condition.
///
/// The condition may be a full condition or only a sub-expression in a larger condition.
///
/// The type that implements this can't be used direct. You need to extract it with a [`VisibilityConditionInspector`]
/// into a [`VisibilityConditionPack`] with [`build_pack`]. Then condition can be evaluated with
/// [`VisibilityConditionPack::evaluate`]
pub(crate) trait VisibilityCondition
{
    /// Inspects the visibility condition.
    fn inspect<'s, 'c: 's, 'b: 'c, 'i: 'b>(&'s self, inspector: &'c mut VisibilityConditionInspector<'c, 'b, 'i>);
}

//-------------------------------------------------------------------------------------------------------------------

//todo: don't require 'static bound
impl<T: VisibilityAttribute + 'static> VisibilityCondition for T
{
    fn inspect<'s, 'c: 's, 'b: 'c, 'i: 'b>(&'s self, inspector: &'c mut VisibilityConditionInspector<'c, 'b, 'i>)
    {
        if let Some(length) = inspector.length()
        {
            *length += 1;
            return;
        }

        let Some(builder) = inspector.builder() else { return; };
        builder.attr_node(attribute_id(self));
    }
}
/*
/// Creates a 'not' visibility condition.
pub fn not<'s, 'a: 's, 'b: 'a>(a: impl VisibilityCondition + 'static) -> impl VisibilityConditionNodeClosure<'s, 'a, 'b>
{
    move |inspector: &'a mut VisibilityConditionInspector<'b>|
    {
        if let Some(length) = inspector.length()
        {
            *length += 1;
            a.inspect(inspector);
            return;
        }

        let Some(builder) = inspector.builder() else { return; };
        builder.not_node();
        a.inspect(inspector);
    }
}

/// Creates an 'and' visibility condition.
pub fn and<'s, 'a: 's, 'b: 'a>(a: impl VisibilityCondition + 'static, b: impl VisibilityCondition + 'static) -> impl VisibilityConditionNodeClosure<'s, 'a, 'b>
{
    move |inspector: &'a mut VisibilityConditionInspector<'b>|
    {
        if let Some(length) = inspector.length()
        {
            *length += 1;
            a.inspect(inspector);
            b.inspect(inspector);
            return;
        }

        let Some(builder) = inspector.builder() else { return; };
        let and_node = builder.increment();
        a.inspect(inspector);

        let Some(builder) = inspector.builder() else { return; };
        builder.and_node(and_node);
        b.inspect(inspector);
    }
}

/// Creates an 'or' visibility condition.
pub fn or<'s, 'a: 's, 'b: 'a>(a: impl VisibilityCondition + 'static, b: impl VisibilityCondition + 'static) -> impl VisibilityConditionNodeClosure<'s, 'a, 'b>
{
    move |inspector: &'a mut VisibilityConditionInspector<'b>|
    {
        if let Some(length) = inspector.length()
        {
            *length += 1;
            a.inspect(inspector);
            b.inspect(inspector);
            return;
        }

        let Some(builder) = inspector.builder() else { return; };
        let or_node = builder.increment();
        a.inspect(inspector);

        let Some(builder) = inspector.builder() else { return; };
        builder.or_node(or_node);
        b.inspect(inspector);
    }
}
 */
//-------------------------------------------------------------------------------------------------------------------

/// Represents a visibility condition node builder.
pub trait VisibilityConditionNodeClosure<'s, 'c: 's, 'b: 'c, 'i: 'b> : Fn(&'c mut VisibilityConditionInspector<'c, 'b, 'i>) + 's {}

impl<'s, 'c: 's, 'b: 'c, 'i: 'b, F> VisibilityConditionNodeClosure<'s, 'c, 'b, 'i> for F
where
    F: Fn(&'c mut VisibilityConditionInspector<'c, 'b, 'i>) + 's
{}

pub type VisibilityConditionNodeClosureT = dyn for<'s, 'c, 'b, 'i> VisibilityConditionNodeClosure<'s, 'c, 'b, 'i, Output = ()>;

impl VisibilityCondition for VisibilityConditionNodeClosureT
{
    fn inspect<'s, 'c: 's, 'b: 'c, 'i: 'b>(&'s self, inspector: &'c mut VisibilityConditionInspector<'c, 'b, 'i>)
    {
        (self)(inspector)
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct VisibilityConditionPackBuilder<'b, 'i: 'b>
{
    next_node: usize,
    nodes: &'i mut [VisibilityConditionNode],
    phantom: PhantomData<&'b()>,
}

impl<'b, 'i: 'b> VisibilityConditionPackBuilder<'b, 'i>
{
    fn new(nodes: &'i mut [VisibilityConditionNode]) -> Self
    {
        Self{ next_node: 0, nodes, phantom: PhantomData::default() }
    }

    /// Increments the node index and returns the previous node index.
    fn increment(&mut self) -> usize
    {
        let prev = self.next_node;
        self.next_node += 1;
        prev
    }

    fn attr_node(&mut self, attr: VisibilityAttributeId)
    {
        self.nodes[self.next_node] = VisibilityConditionNode::Attr(attr);
        self.next_node += 1;
    }

    fn not_node(&mut self)
    {
        self.nodes[self.next_node] = VisibilityConditionNode::Not(self.next_node + 1);
        self.next_node += 1;
    }

    fn and_node(&mut self, node: usize)
    {
        let left = node + 1;
        let right = self.next_node;
        self.nodes[node] = VisibilityConditionNode::And(left, right);
    }

    fn or_node(&mut self, node: usize)
    {
        let left = node + 1;
        let right = self.next_node;
        self.nodes[node] = VisibilityConditionNode::Or(left, right);
    }
}

//-------------------------------------------------------------------------------------------------------------------

enum VisibilityConditionInspector<'c, 'b: 'c, 'i: 'b>
{
    Ignored(PhantomData<&'c()>),
    ComputeLength(usize),
    AddNode(VisibilityConditionPackBuilder<'b, 'i>),
}

impl<'c, 'b: 'c, 'i: 'b> VisibilityConditionInspector<'c, 'b, 'i>
{
    fn length(&mut self) -> Option<&mut usize>
    {
        let Self::ComputeLength(length) = self else { return None; };
        Some(length)
    }

    fn builder(&mut self) -> Option<&mut VisibilityConditionPackBuilder<'b, 'i>>
    {
        let Self::AddNode(builder) = self else { return None; };
        Some(builder)
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_pack(condition: impl VisibilityCondition + 'static) -> VisibilityConditionPack
{
    // length
    let mut inspector = VisibilityConditionInspector::ComputeLength(0);
    condition.inspect(&mut inspector);
    let Some(len) = inspector.length() else { unreachable!(); };

    // pack
    VisibilityConditionPack::new_with(*len,
        |nodes|
        {
            let builder = VisibilityConditionPackBuilder::new(nodes);
            let mut inspector = VisibilityConditionInspector::AddNode(builder);
            condition.inspect(&mut inspector);
        }
    )
}

//-------------------------------------------------------------------------------------------------------------------

/// Max number of nodes for non-allocating [`VisibilityConditionPack`]s.
pub(crate) const SMALL_PACK_LEN: usize = 3;

/// A type-erased visibility condition.
///
/// Constructing a pack only requires allocations if the condition contains more than [`SMALL_PACK_LEN`] nodes.
///
/// Examples:
/// - 1 node: `VisibleTo(Global)`
/// - 2 nodes: `VisibleTo(!InABush)`
/// - 3 nodes: `VisibleTo(IsFast && IsSmall)`
/// - 4 nodes: `VisibleTo(IsSwimming && !WearingSwimsuit)`
#[derive(Debug, Clone, Hash)]
pub(crate) enum VisibilityConditionPack
{
    Small{
        condition: [VisibilityConditionNode; SMALL_PACK_LEN],
    },
    Large{
        condition: Arc<[VisibilityConditionNode]>,
    },
}

impl VisibilityConditionPack
{
    /// Makes a new pack with the given condition writer.
    pub(crate) fn new_with(len: usize, writer: impl FnOnce(&mut [VisibilityConditionNode])) -> Self
    {
        if len <= SMALL_PACK_LEN
        {
            let mut pack = Self::Small{ condition: Default::default() };
            let Self::Small{ mut condition } = &mut pack
            else
            {
                // SAFETY: We just made this.
                unsafe { std::hint::unreachable_unchecked() }
            };
            writer(condition.as_mut_slice());
            pack
        }
        else
        {
            let mut condition = Vec::with_capacity(len);
            condition.resize(len, VisibilityConditionNode::default());
            writer(condition.as_mut_slice());
            Self::Large{ condition: Arc::from(condition) }
        }
    }

    /// Gets the condition id.
    pub(crate) fn id(&self) -> VisibilityConditionId
    {
        let mut hasher = SipHasher13::new();
        self.hash(&mut hasher);
        let id = hasher.finish128();

        VisibilityConditionId(id.into())
    }

    /// Iterates attributes within the condition tree.
    pub(crate) fn iter_attributes(&self) -> impl Iterator<Item = VisibilityAttributeId> + '_
    {
        let filter = |n: &VisibilityConditionNode| -> Option<VisibilityAttributeId>
        {
            let VisibilityConditionNode::Attr(attr) = n else { return None; };
            Some(*attr)
        };
        match self
        {
            Self::Small{ condition } =>
            {
                condition.iter().filter_map(filter)
            }
            Self::Large{ condition } =>
            {
                condition.iter().filter_map(filter)
            }
        }
    }

    /// Evaluates the condition tree with the attribute evaluator.
    ///
    /// Modifiers are automatically evaluated. The evaluator only checks if the given attribute is known.
    pub(crate) fn evaluate(&self, evaluator: impl Fn(VisibilityAttributeId) -> bool) -> bool
    {
        match self
        {
            Self::Small{ condition } =>
            {
                evaluate(&evaluator, condition.as_slice(), 0)
            }
            Self::Large{ condition } =>
            {
                evaluate(&evaluator, condition, 0)
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component that records the [`VisibilityCondition`] for an entity.
//todo: add Component
#[derive(Debug, Clone)]
pub struct VisibleTo
{
    pub(crate) pack: VisibilityConditionPack,
}

impl VisibleTo
{
    /// Makes a new `VisibleTo` component.
    pub fn new(condition: impl VisibilityCondition + 'static) -> Self
    {
        Self{ pack: build_pack(condition) }
    }

    /// Gets the id of the validity condition.
    ///
    /// Note that this requires hashing the internal condition, which may be expensive.
    /// We don't cache the id here since it is 16 bytes.
    pub fn condition_id(&self) -> VisibilityConditionId
    {
        self.pack.id()
    }
}

impl PartialEq for VisibleTo
{
    fn eq(&self, other: &Self) -> bool
    {
        self.condition_id() == other.condition_id()
    }
}
impl Eq for VisibleTo {}

//-------------------------------------------------------------------------------------------------------------------
