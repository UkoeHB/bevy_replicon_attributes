//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::{Component, Deref};
use bevy_replicon::prelude::Replication;
use siphasher::sip128::{Hasher128, SipHasher13};
use smallvec::SmallVec;

//standard shortcuts
use std::any::TypeId;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

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
pub struct VisibilityConditionBuilder
{
    nodes: SmallVec<[VisibilityConditionNode; SMALL_PACK_LEN]>,
}

impl VisibilityConditionBuilder
{
    /// Creates a new condition builder.
    pub(crate) fn new() -> Self
    {
        Self{ nodes: SmallVec::default() }
    }

    /// Pushes an empty node which will be set later.
    ///
    /// Allows defining how many `extra` nodes are associated with this node, to improve reallocation accuracy.
    pub(crate) fn push_empty(&mut self, extra: usize) -> usize
    {
        let position = self.nodes.len();
        self.nodes.reserve(extra + 1);
        self.nodes.push(VisibilityConditionNode::Empty);
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
        let next_node = self.nodes.len() + 1;
        self.nodes.reserve(2);
        self.nodes.push(VisibilityConditionNode::Not(next_node));
    }

    /// Sets an AND node at its branch root position.
    ///
    /// Assumes the next node to be inserted will be the start of the AND expression's right branch.
    ///
    /// Panics if the AND node position was not inserted with [`Self::push_empty`].
    pub(crate) fn set_and_node(&mut self, node: usize)
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
    pub(crate) fn set_or_node(&mut self, node: usize)
    {
        let left = node + 1;
        let right = self.nodes.len();
        self.nodes[node] = VisibilityConditionNode::Or(left, right);
    }

    /// Pushes a condition branch into the tree.
    pub(crate) fn push_branch(&mut self, branch: &[VisibilityConditionNode])
    {
        self.nodes.reserve(branch.len());
        let len = self.nodes.len();

        for mut node in branch.iter().copied()
        {
            match &mut node
            {
                VisibilityConditionNode::Empty     => (),
                VisibilityConditionNode::Attr(_)   => (),
                VisibilityConditionNode::Not(a)    => { *a += len; },
                VisibilityConditionNode::And(a, b) => { *a += len; *b += len; },
                VisibilityConditionNode::Or(a, b)  => { *a += len; *b += len; },
            }
            self.nodes.push(node);
        }
    }

    /// Takes the internal nodes.
    pub(crate) fn take(self) -> SmallVec<[VisibilityConditionNode; SMALL_PACK_LEN]>
    {
        self.nodes
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
                and(into_condition!($a), into_condition!($a))
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

/// Syntax sugar for the bundle `(Replication, vis!([your visibility condition]))`.
/// 
/// Semantically, the bundle produced here replicates an entity to clients that match the specified visibility condition.
/// 
/// Example:
/**
```rust
commands.spawn((PlayerInventory, replicate_to!(IsClient(client_id))));
```
*/
#[macro_export] macro_rules! replicate_to
{
    ($($condition:tt)*) =>
    {
        (Replication, vis!($($condition)*))
    };
}

//-------------------------------------------------------------------------------------------------------------------
