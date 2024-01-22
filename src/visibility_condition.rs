//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::{Component, Deref, DerefMut};
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
    let a = current_node + 1;
    match condition[current_node]
    {
        VisibilityConditionNode::Empty => { true }
        VisibilityConditionNode::Attr(attr) => (inspector)(attr),
        VisibilityConditionNode::Not        => !evaluate(inspector, condition, a),
        VisibilityConditionNode::And(b)     => evaluate(inspector, condition, a) && evaluate(inspector, condition, b),
        VisibilityConditionNode::Or(b)      => evaluate(inspector, condition, a) || evaluate(inspector, condition, b),
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
    /// Next node is child.
    Not,
    /// Next node is left branch. Records node of right branch.
    And(usize),
    /// Next node is left branch. Records node of right branch.
    Or(usize),
}

impl VisibilityConditionNode
{
    /// Returns true if two nodes that appear at the same position within subslices of larger patterns are equivalent.
    ///
    /// Non-root nodes are equivalent if they are the same type and point to the same nodes within those subslices.
    fn equivalent(&self, self_ref: usize, other: &Self, other_ref: usize) -> bool
    {
        match self
        {
            Self::Empty =>
            {
                let Self::Empty = other else { return false; };
                true
            }
            Self::Attr(attr) =>
            {
                let Self::Attr(attr_other) = other else { return false; };
                attr == attr_other
            }
            Self::Not =>
            {
                let Self::Not = other else { return false; };
                true
            }
            Self::And(b) =>
            {
                let Self::And(b_other) = other else { return false; };
                (b - self_ref) == (b_other - other_ref)
            }
            Self::Or(b) =>
            {
                let Self::Or(b_other) = other else { return false; };
                (b - self_ref) == (b_other - other_ref)
            }
        }
    }
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
/// Note that empty conditions always evaluate to `true`.
///
/// See also [`Visibility`].
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
    /// Makes a new condition with one empty node.
    pub fn empty() -> Self
    {
        Self::Small(SmallVec::from_slice(&[VisibilityConditionNode::Empty]))
    }

    /// Makes a new condition with the given condition constructor.
    ///
    /// The final condition will be consolidated (empty nodes removed and expressions simplified).
    pub fn new(condition: impl IntoVisibilityCondition) -> Self
    {
        let builder = VisibilityConditionBuilder::new();
        let final_builder = condition.build(builder);
        Self::from(final_builder)
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

    /// Checks if the current condition is empty.
    ///
    /// Note that empty conditions always evaluate to `true`.
    pub fn is_empty(&self) -> bool
    {
        match self
        {
            Self::Small(condition) => condition.len() == 1 && matches!(condition[0], VisibilityConditionNode::Empty),
            Self::Large(condition) => condition.len() == 1 && matches!(condition[0], VisibilityConditionNode::Empty),
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

    /// Extends self with an AND relationship with another visibility condition.
    ///
    /// The final condition will be consolidated (empty nodes removed and expressions simplified).
    ///
    /// Example:
    /**
    ```rust
    let mut a = vis!(or(A, B));
    a.and(and(C, not(D)));
    assert!(a == vis!(and(or(A, B), and(C, not(D)))));
    ```
    */
    pub fn and(&mut self, other: impl IntoVisibilityCondition)
    {
        *self = Self::new(and(self.clone(), other));
    }

    /// Extends self with an OR relationship with another visibility condition.
    ///
    /// The final condition will be consolidated (empty nodes removed and expressions simplified).
    ///
    /// Example:
    /**
    ```rust
    let mut a = vis!(not(A));
    a.or(B);
    assert!(a == vis!(or(not(A), B)));
    ```
    */
    pub fn or(&mut self, other: impl IntoVisibilityCondition)
    {
        *self = Self::new(or(self.clone(), other));
    }

    /// Replaces instances of a pattern in the current visibility condition with a new condition branch.
    ///
    /// The final condition will be consolidated (empty nodes removed and expressions simplified).
    ///
    /// Returns the number of pattern instances replaced.
    /// Does nothing if the input pattern is empty.
    ///
    /// Examples:
    /**
    ```rust
    let mut a = vis!(and(A, B));
    a.replace(B, not(B));
    assert!(a == vis!(and(A, not(B))));

    a.replace(and(A, not(B)), C);
    assert!(a == vis!(C));
    ```
    */
    pub fn replace(&mut self, pattern: impl IntoVisibilityCondition, replacement: impl IntoVisibilityCondition) -> usize
    {
        let pattern = Self::new(pattern);
        let pattern = pattern.as_slice();
        if pattern.len() == 0 { return 0; }

        self.replace_with(
            pattern.len(),
            |base, pos, node| pattern[pos].equivalent(0, node, base),
            replacement,
        )
    }

    /// Replaces attribute nodes of a certain type in the current visibility condition with a new condition branch.
    ///
    /// This is a type-only search and replace. Attributes of the same type but different inner ids will be replaced.
    /// For replacement that includes inner id checks, use [`Self::replace`].
    ///
    /// The final condition will be consolidated (empty nodes removed and expressions simplified).
    ///
    /// Returns the number of attribute nodes replaced.
    ///
    /// Examples:
    /**
    ```rust
    let mut a = vis!(and(A(0), not(A(1))));
    a.replace_type::<A>(B);
    assert!(a == vis!(and(B, not(B))));

    let mut loc = vis!(Location(20, 40));
    loc.replace_type::<Location>(Location(22, 45));
    assert!(loc == vis!(Location(22, 45)));
    ```
    */
    pub fn replace_type<T: VisibilityAttribute>(&mut self, replacement: impl IntoVisibilityCondition) -> usize
    {
        let comparison = VisibilityAttributeId::new::<T>(0u64).type_id();
        self.replace_with(
            1,
            |_, _, node|
            {
                let VisibilityConditionNode::Attr(attr) = node else { return false; };
                attr.type_id() == comparison
            },
            replacement,
        )
    }

    /// Removes instances of a pattern in the current visibility condition.
    ///
    /// The final condition will be consolidated (empty nodes removed and expressions simplified).
    ///
    /// Returns the number of pattern instances removed.
    /// Does nothing if the input pattern is empty.
    ///
    /// Examples:
    /**
    ```rust
    let mut a = vis!(and(A, B));
    a.remove(B);
    assert!(a == vis!(A));

    let mut b = vis!(or(A, not(B)));
    b.remove(or(A, not(B)));
    assert!(b == vis!());
    ```
    */
    pub fn remove(&mut self, pattern: impl IntoVisibilityCondition) -> usize
    {
        self.replace(pattern, empty())
    }

    /// Removes attribute nodes of a certain type in the current visibility condition.
    ///
    /// The final condition will be consolidated (empty nodes removed and expressions simplified).
    ///
    /// This is a type-only search and remove. Attributes of the same type but different inner ids will be removed.
    /// For removal that includes inner id checks, use [`Self::remove`].
    ///
    /// Returns the number of pattern instances removed.
    ///
    /// Examples:
    /**
    ```rust
    let mut a = vis!(and(A(0), not(A(1))));
    a.remove_type::<A>();
    assert!(a == vis!());

    let mut b = vis!(or(A(1), B(2)));
    b.remove_type::<A>();
    assert!(b == vis!(B(2)));
    ```
    */
    pub fn remove_type<T: VisibilityAttribute>(&mut self) -> usize
    {
        self.replace_type::<T>(empty())
    }

    /// Makes a new condition from a builder.
    fn from(builder: VisibilityConditionBuilder) -> Self
    {
        // extract the node tree
        let mut condition = builder.consolidate_and_take();

        // simplify
        condition.shrink_to_fit();

        // save the result
        if !condition.spilled()
        {
            Self::Small(condition)
        }
        else
        {
            Self::Large(Arc::from(condition.into_vec()))
        }
    }

    /// Replaces sections of the existing condition with a replacement condition.
    fn replace_with(
        &mut self,
        pattern_len: usize,
        // fn(position in ref node's condition, position in pattern, ref node) -> equivalence
        pattern_checker: impl Fn(usize, usize, &VisibilityConditionNode) -> bool,
        replacement: impl IntoVisibilityCondition,
    ) -> usize
    {
        if pattern_len == 0 { return 0; }

        // build
        let replacement = Self::new(replacement);
        let replacement = replacement.as_slice();
        let delta = (replacement.len() as i32) - (pattern_len as i32);
        let mut builder = VisibilityConditionBuilder::new();

        let slice = self.as_slice();
        let mut count = 0;
        let mut dead_zone_start = 0;
        let mut test_zone_start = 0;
        let mut test_zone_end = 0;
        let mut scan_end = pattern_len;

        while scan_end <= slice.len()
        {
            // check for node equivalence
            while test_zone_end < pattern_len
            {
                // check the node
                if !pattern_checker(test_zone_start, test_zone_end, &slice[test_zone_start + test_zone_end]) { break; }

                // nodes are equivalent, advance to the next node
                test_zone_end += 1;
            }

            // handle full pattern match
            if test_zone_end == pattern_len
            {
                // write pre-scan slice to builder
                builder.push_branch(dead_zone_start, &slice[dead_zone_start..test_zone_start]);

                // write replacement slice to builder
                builder.prep_branch_replacement(delta);
                builder.push_branch(0, replacement);

                // bump to the next range of size 'pattern length'
                count += 1;
                dead_zone_start = scan_end;
                test_zone_start = scan_end;
                scan_end += pattern_len;
                test_zone_end = 0;
            }
            else
            {
                // bump up one notch
                test_zone_start += 1;
                scan_end += 1;
                test_zone_end = 0;
            }
        }

        if count > 0
        {
            builder.push_branch(dead_zone_start, &slice[dead_zone_start..test_zone_start]);
            *self = Self::from(builder);
        }

        count
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
        builder.push_branch(0, self.as_slice());
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
#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct Visibility(VisibilityCondition);

impl Visibility
{
    /// Makes a new `Visibility` component.
    pub fn new(condition: impl IntoVisibilityCondition) -> Self
    {
        Self(VisibilityCondition::new(condition))
    }
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
