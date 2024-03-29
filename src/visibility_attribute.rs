//local shortcuts
use crate::*;

//third-party shortcuts

//standard shortcuts
use std::any::TypeId;
use std::hash::Hash;

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

    /// Gets the attribute's type id.
    pub fn type_id(self) -> TypeId
    {
        self.0
    }

    /// Gets the attribute's inner id.
    pub fn inner_id(self) -> u64
    {
        self.1
    }
}

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
    /// If your attribute contains non-type information (e.g. a client id), then you should manually implement this.
    ///
    /// Note that ids are domain-separated by attribute type, so you can safely use the full `u64` range to define your
    /// inner id.
    fn inner_attribute_id(&self) -> u64;

    /// Returns the id of the attribute.
    ///
    /// The id is a concatenation of the attribute's type id and its inner id.
    fn attribute_id(&self) -> VisibilityAttributeId
    {
        VisibilityAttributeId::new::<Self>(self.inner_attribute_id())
    }
}

impl<T: VisibilityAttribute> IntoVisibilityCondition for T
{
    fn build(self, mut builder: VisibilityConditionBuilder) -> VisibilityConditionBuilder
    {
        builder.push_attr_node(self.attribute_id());
        builder
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Implemented by the derive for [`VisibilityAttribute`].
///
/// This trait requires `Default` and `PartialEq` to enforce that only default-constructed objects can be
/// assigned a default visibility attribute id.
///
/// [`VisibilityAttribute::inner_attribute_id`] will panic on types that implement this if the attribute does
/// not equal its default value.
/// If that happens, you should manually implement [`VisibilityAttribute`] and define an appropriate inner attribute id
/// for your type.
pub trait DefaultVisibilityAttribute: Default + PartialEq + 'static {}

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
