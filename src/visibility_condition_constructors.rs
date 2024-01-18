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

/// Represents a visibility condition expression builder.
pub trait VisibilityConditionExpression: FnOnce(VisibilityConditionBuilder) -> VisibilityConditionBuilder + 'static
{}

impl<F> VisibilityConditionExpression for F
where
    F: FnOnce(VisibilityConditionBuilder) -> VisibilityConditionBuilder + 'static
{}

pub type DummyVisClosure = fn(VisibilityConditionBuilder) -> VisibilityConditionBuilder;

//-------------------------------------------------------------------------------------------------------------------

/// Wrapper enum that translates types into a type that implements [`IntoVisibilityCondition`].
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

//-------------------------------------------------------------------------------------------------------------------

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

//-------------------------------------------------------------------------------------------------------------------

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

//-------------------------------------------------------------------------------------------------------------------

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
