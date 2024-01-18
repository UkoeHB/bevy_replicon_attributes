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
/// - 1 node: `VisibilityCondition::new(attr(Global))`
/// - 2 nodes: `VisibilityCondition::new(not(attr(InABush)))`
/// - 3 nodes: `VisibilityCondition::new(and(attr(IsFast), attr(IsSmall))`
/// - 4 nodes: `VisibilityCondition::new(and(attr(IsSwimming), not(attr(WearingSwimsuit))))`
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

    /// Evaluates the condition tree with an attribute evaluator.
    ///
    /// The evaluator should check if a given attribute is known. Modifiers (not/and/or) are automatically evaluated.
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
