//local shortcuts
use crate::*;

//third-party shortcuts
use smallvec::SmallVec;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Represents a visibility condition expression builder.
trait VisibilityConditionExpression: FnOnce(VisibilityConditionBuilder) -> VisibilityConditionBuilder + 'static
{}

impl<F> VisibilityConditionExpression for F
where
    F: FnOnce(VisibilityConditionBuilder) -> VisibilityConditionBuilder + 'static
{}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Wrapper that allows [`IntoVisibilityCondition`] to be implemented for condition expressions.
struct VisibilityConditionWrapper<E>(E)
where
    E: VisibilityConditionExpression;

impl<E> From<E> for VisibilityConditionWrapper<E>
where
    E: VisibilityConditionExpression,
{
    fn from(e: E) -> Self
    {
        Self(e)
    }
}

impl<E> IntoVisibilityCondition for VisibilityConditionWrapper<E>
where
    E: VisibilityConditionExpression,
{
    fn build(self, builder: VisibilityConditionBuilder) -> VisibilityConditionBuilder
    {
        (self.0)(builder)
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Visibility condition builder.
#[derive(Debug)]
pub struct VisibilityConditionBuilder
{
    nodes: SmallVec<[VisibilityConditionNode; SMALL_PACK_LEN]>,
    num_empty: usize,
}

impl VisibilityConditionBuilder
{
    /// Creates a new condition builder.
    pub(crate) fn new() -> Self
    {
        Self{ nodes: SmallVec::default(), num_empty: 0 }
    }

    /// Pushes an empty node which will be set later.
    ///
    /// Allows defining how many `extra` nodes are associated with this node, to improve reallocation accuracy.
    pub(crate) fn push_empty(&mut self, extra: usize) -> usize
    {
        let position = self.nodes.len();
        self.nodes.reserve(extra + 1);
        self.nodes.push(VisibilityConditionNode::Empty);
        self.num_empty += 1;
        position
    }

    /// Adds an ATTRIBUTE node to the end of the condition.
    pub(crate) fn push_attr_node(&mut self, attr: VisibilityAttributeId)
    {
        self.nodes.push(VisibilityConditionNode::Attr(attr));
    }

    /// Adds a NOT node to the end of the condition.
    ///
    /// Assumes the next node to be inserted will be the start of the OR expression's child branch.
    pub(crate) fn push_not_node(&mut self)
    {
        self.nodes.reserve(2);
        self.nodes.push(VisibilityConditionNode::Not);
    }

    /// Sets an AND node at its branch root position.
    ///
    /// Assumes the next node to be inserted will be the start of the AND expression's right branch.
    ///
    /// Panics if the AND node position was not inserted with [`Self::push_empty`].
    pub(crate) fn set_and_node(&mut self, node: usize)
    {
        let right = self.nodes.len();
        self.nodes[node] = VisibilityConditionNode::And(right);
        self.num_empty -= 1;
    }

    /// Sets an OR node at its branch root position.
    ///
    /// Assumes the next node to be inserted will be the start of the OR expression's right branch.
    ///
    /// Panics if the OR node position was not inserted with [`Self::push_empty`].
    pub(crate) fn set_or_node(&mut self, node: usize)
    {
        let right = self.nodes.len();
        self.nodes[node] = VisibilityConditionNode::Or(right);
        self.num_empty -= 1;
    }

    /// Pushes a condition branch into the tree.
    ///
    /// The inserted branch may come from a section of another condition, in which case you should define the `root`
    /// of the branch being inserted to equal the position of the first node in that branch.
    pub(crate) fn push_branch(&mut self, root: usize, branch: &[VisibilityConditionNode])
    {
        if branch.len() == 0 { return; }

        self.nodes.reserve(branch.len());
        let len = self.nodes.len();

        for mut node in branch.iter().copied()
        {
            match &mut node
            {
                VisibilityConditionNode::Empty   => { self.num_empty += 1; },
                VisibilityConditionNode::Attr(_) => (),
                VisibilityConditionNode::Not     => (),
                VisibilityConditionNode::And(b)  |
                VisibilityConditionNode::Or(b)   => { *b -= root; *b += len; }
            }
            self.nodes.push(node);
        }
    }

    /// Repairs existing node references with the assumption that we are rebuilding a condition, and
    /// a segment of the old condition will be replaced by a new segment (which will be inserted to the end of the builder).
    ///
    /// The `replacement_delta` equals the new segment minus old segment length.
    pub(crate) fn prep_branch_replacement(&mut self, replacement_delta: i32)
    {
        let len = self.nodes.len();

        for node in self.nodes.iter_mut()
        {
            match node
            {
                VisibilityConditionNode::Empty   => (),
                VisibilityConditionNode::Attr(_) => (),
                VisibilityConditionNode::Not     => (),
                VisibilityConditionNode::And(b)  |
                VisibilityConditionNode::Or(b)   =>
                {
                    // only correct right branch ptr that points past the starting position of the replacement
                    if *b > len { *b = (*b as i32 + replacement_delta) as usize; }
                }
            }
        }
    }

    /// Consolidates the condition by removing empty nodes and simplifying expressions.
    ///
    /// Returns the consolidated internal node tree.
    ///
    /// An empty node is inserted if the condition is empty. We want future compositions using this condition
    /// to correctly consolidate, so we need it to have an empty node to avoid broken expressions.
    pub(crate) fn consolidate_and_take(self) -> SmallVec<[VisibilityConditionNode; SMALL_PACK_LEN]>
    {
        if self.nodes.len() == 0 { return SmallVec::from_slice(&[VisibilityConditionNode::Empty]); }
        if self.nodes.len() == 1 || self.num_empty == 0 { return self.nodes; }

        // set all invalid branches to empty
        fn node_recursion(
            nodes        : SmallVec<[VisibilityConditionNode; SMALL_PACK_LEN]>,
            current_node : usize,
        ) -> (bool, SmallVec<[VisibilityConditionNode; SMALL_PACK_LEN]>, usize)
        {
            match nodes[current_node]
            {
                VisibilityConditionNode::Empty   => (true, nodes, 1),
                VisibilityConditionNode::Attr(_) => (false, nodes, 0),
                VisibilityConditionNode::Not     =>
                {
                    // recurse child branch
                    let (is_empty, mut nodes, num_empty) = node_recursion(nodes, current_node + 1);

                    // if child is non-empty, then do nothing
                    if !is_empty { return (false, nodes, num_empty); }

                    // invalidate current node
                    nodes[current_node] = VisibilityConditionNode::Empty;
                    (true, nodes, num_empty + 1)
                }
                VisibilityConditionNode::And(b) |
                VisibilityConditionNode::Or(b)  =>
                {
                    // recurse branches
                    let (left_is_empty, nodes, num_empty_left) = node_recursion(nodes, current_node + 1);
                    let (right_is_empty, mut nodes, num_empty_right) = node_recursion(nodes, b);

                    // if both branches are non-empty, then adjust the left branch reference
                    if !left_is_empty && !right_is_empty
                    {
                        match &mut nodes[current_node]
                        {
                            VisibilityConditionNode::And(b) |
                            VisibilityConditionNode::Or(b)  => { *b -= num_empty_left; }
                            _ => { unreachable!(); }
                        }
                        return (false, nodes, num_empty_left + num_empty_right);
                    }

                    // invalidate current node
                    // - This node is only completely empty if both branches are empty. If only one branch is empty, then
                    //   the non-empty branch 'takes over' the position of this node within the parent.
                    nodes[current_node] = VisibilityConditionNode::Empty;
                    (left_is_empty && right_is_empty, nodes, num_empty_left + num_empty_right + 1)
                }
            }
        }

        let (empty_tree, mut nodes, num_empty) = node_recursion(self.nodes, 0);
        if empty_tree { return SmallVec::from_slice(&[VisibilityConditionNode::Empty]); }

        // repair branch references based on empty nodes, and shift elements left
        let mut empty_count = 0;

        for idx in 0..nodes.len()
        {
            match &mut nodes[idx]
            {
                VisibilityConditionNode::Empty   => { empty_count += 1; continue; },
                VisibilityConditionNode::Attr(_) => (),
                VisibilityConditionNode::Not     => (),
                // note: we incorporated left-branch empty slots within the recursion
                VisibilityConditionNode::And(b)  |
                VisibilityConditionNode::Or(b)   => { *b -= empty_count; }
            }
            nodes[idx - empty_count] = nodes[idx];
        }
        debug_assert_eq!(num_empty, empty_count);

        nodes.truncate(nodes.len() - empty_count);
        nodes
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

/// Creates an EMPTY visibility condition.
///
/**
```rust
let condition = VisibilityCondition::new(empty());
``` 
*/
pub fn empty() -> impl IntoVisibilityCondition
{
    VisibilityConditionWrapper::from(
        |mut builder: VisibilityConditionBuilder| -> VisibilityConditionBuilder
        {
            builder.push_empty(0);
            builder
        }
    )
}

//-------------------------------------------------------------------------------------------------------------------

/// Creates a NOT visibility condition.
///
/**
```rust
#[derive(VisibilityAttribute, Default, PartialEq)]
struct A;

let condition = VisibilityCondition::new(not(A));
``` 
*/
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

//-------------------------------------------------------------------------------------------------------------------

/// Creates an AND visibility condition.
///
/**
```rust
#[derive(VisibilityAttribute, Default, PartialEq)]
struct A;
#[derive(VisibilityAttribute, Default, PartialEq)]
struct B;

let condition = VisibilityCondition::new(and(A, B));
``` 
*/
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

//-------------------------------------------------------------------------------------------------------------------

/// Creates an OR visibility condition.
///
/**
```rust
#[derive(VisibilityAttribute, Default, PartialEq)]
struct A;
#[derive(VisibilityAttribute, Default, PartialEq)]
struct B;

let condition = VisibilityCondition::new(or(A, B));
``` 
*/
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

/// Syntax sugar for `and(A, and(B, C))` etc.
#[macro_export] macro_rules! all
{
    () => { empty() };
    ($condition:expr) => { $condition };
    ($condition:expr, $($remaining:tt)*) =>
    {
        and($condition, all!($($remaining)*))
    };
}

//-------------------------------------------------------------------------------------------------------------------

/// Syntax sugar for `or(A, or(B, C))` etc.
#[macro_export] macro_rules! any
{
    () => { empty() };
    ($condition:expr) => { $condition };
    ($condition:expr, $($remaining:tt)*) =>
    {
        or($condition, any!($($remaining)*))
    };
}

//-------------------------------------------------------------------------------------------------------------------

/// Syntax sugar for `not(or(A, or(B, C)))` etc.
#[macro_export] macro_rules! none
{
    () => { empty() };
    ($($condition:tt)*) =>
    {
        not(any!($($condition)*))
    };
}

//-------------------------------------------------------------------------------------------------------------------

/// Translates a token sequence into a type that implements [`IntoVisibilityCondition`].
#[macro_export] macro_rules! into_condition
{
    /*($a:expr + $b:expr) =>
    {
        and(into_condition!(a), into_condition!(b))
    };
    ($($a:tt),+) =>
    {
        {
            $(
                or(into_condition!($a), into_condition!($a))
            )*
        }
    };
    //this one works but for consistency it is disabled until and/or also work
    (!$($inner:tt)+) =>
    {
        not(into_condition!($($inner)*))
    };*/
    ($attribute:expr) =>
    {
        $attribute
    };
    () =>
    {
        empty()
    };
}

//-------------------------------------------------------------------------------------------------------------------

/// Syntax sugar for [`Visibility::new`].
#[macro_export] macro_rules! vis
{
    ($($condition:tt)*) =>
    {
        Visibility::new(into_condition!($($condition)*))
    };
}

//-------------------------------------------------------------------------------------------------------------------
