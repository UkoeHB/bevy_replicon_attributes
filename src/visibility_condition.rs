//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::{Component, Deref};
use siphasher::sip128::{Hasher128, SipHasher13};
use smallvec::SmallVec;

//standard shortcuts
use std::hash::Hash;
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------
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
        { tracing::error!("encountered empty node when evaluating condition tree"); true }
        VisibilityConditionNode::Attr(attr) => (inspector)(attr),
        VisibilityConditionNode::Not(a)     => !evaluate(inspector, condition, a),
        VisibilityConditionNode::And(a, b)  => evaluate(inspector, condition, a) && evaluate(inspector, condition, b),
        VisibilityConditionNode::Or(a, b)   => evaluate(inspector, condition, a) || evaluate(inspector, condition, b),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Id associated with a visibility condition.
///
/// This is used to differentiate visibility conditions within the attribute engine's internal maps.
/// The id has 64 bits of collision resistance, which should be adequate for the vast majority of use-cases.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct VisibilityConditionId(u128);

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

/// Max number of nodes for non-allocating [`VisibilityCondition`]s.
pub const SMALL_PACK_LEN: usize = 3;

//-------------------------------------------------------------------------------------------------------------------

/// A type-erased visibility condition.
///
/// Constructing a condition only requires allocations if the condition contains more than [`SMALL_PACK_LEN`] nodes.
/// Cloning a condition will *not* allocate.
///
/// Use [`Self::evaluate`] to evaluate the condition.
///
/// Examples:
/// - 1 node: `VisibilityCondition::new(Global)`
/// - 2 nodes: `VisibilityCondition::new(not(InABush))`
/// - 3 nodes: `VisibilityCondition::new(and(IsFast, IsSmall)`
/// - 4 nodes: `VisibilityCondition::new(and(IsSwimming, not(WearingSwimsuit)))`
#[derive(Debug, Clone, Hash)]
pub enum VisibilityCondition
{
    Small(SmallVec<[VisibilityConditionNode; SMALL_PACK_LEN]>),
    Large(Arc<[VisibilityConditionNode]>),
}

impl VisibilityCondition
{
    /// Makes a new condition with the given condition constructor.
    pub fn new(condition: impl IntoVisibilityCondition) -> Self
    {
        let builder = VisibilityConditionBuilder::new();
        let final_builder = condition.build(builder);
        let condition = final_builder.take();
        //todo: consolidate empty nodes

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

    /// Iterates attributes referenced in the condition tree.
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

    /// Accesses the inner condition tree as a sequence of nodes.
    pub fn as_slice(&self) -> &[VisibilityConditionNode]
    {
        match self
        {
            Self::Small(condition) => condition.as_slice(),
            Self::Large(condition) => condition,
        }
    }

    /// Evaluates the condition tree with an attribute evaluator.
    ///
    /// The evaluator should check if a given attribute is known. Modifiers (not/and/or) are automatically evaluated.
    ///
    /// Returns `true` for empty conditions.
    pub fn evaluate(&self, evaluator: impl Fn(VisibilityAttributeId) -> bool) -> bool
    {
        let slice = self.as_slice();
        if slice.len() == 0 { return true; }
        evaluate(&evaluator, slice, 0)
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

impl IntoVisibilityCondition for VisibilityCondition
{
    fn build(self, mut builder: VisibilityConditionBuilder) -> VisibilityConditionBuilder
    {
        builder.push_branch(self.as_slice());
        builder
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component that records the visibility for an entity.
///
/// An empty visibility condition always evaluates to `true`. This way if an entity has no `Visibility` component it will
/// be visible to no clients, if it has an empty `Visibility` it will be visible to all clients, and if it has a non-empty
/// `Visibility` then it will be visible to clients that match the condition.
///
/// Semantically, an empty condition matches 'anything', and a non-empty condition is equivalent to `and(ANYTHING, condition)`.
///
/// Derefs to a [`VisibilityCondition`].
#[derive(Component, Debug, Clone, Deref)]
pub struct Visibility(VisibilityCondition);

impl Visibility
{
    /// Makes a new `Visibility` component.
    pub fn new(condition: impl IntoVisibilityCondition) -> Self
    {
        Self(VisibilityCondition::new(condition))
    }

    /// Extends self with an AND relationship with another visibility condition.
    ///
    /// Example:
    /**
    let mut a = vis!(or(A, B));
    a.and(and(C, not(D)));
    // a == vis!(and(or(A, B), and(C, not(D))))
    */
    pub fn and(&mut self, other: impl IntoVisibilityCondition)
    {
        *self = Self::new(and(self.0.clone(), other));
    }

    /// Extends self with an OR relationship with another visibility condition.
    ///
    /// Example:
    /**
    let mut a = vis!(not(A));
    a.or(B);
    // a == vis!(or(not(A), B))
    */
    pub fn or(&mut self, other: impl IntoVisibilityCondition)
    {
        *self = Self::new(or(self.0.clone(), other));
    }

    //todo: replace(a, b) to replace a specific pattern
    //todo: replace_type<T>(a)
    //todo: remove(a) to remove a specific pattern and simplify the condition
    //todo: remove_type<T>()
}

impl PartialEq for Visibility
{
    fn eq(&self, other: &Self) -> bool
    {
        self.0 == other.0
    }
}
impl Eq for Visibility {}

impl IntoVisibilityCondition for Visibility
{
    fn build(self, builder: VisibilityConditionBuilder) -> VisibilityConditionBuilder
    {
        self.0.build(builder)
    }
}

//-------------------------------------------------------------------------------------------------------------------
