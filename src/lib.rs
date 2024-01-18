//documentation
#![doc = include_str!("../README.md")]
#![allow(unused_imports)]
use crate as bevy_replicon_attributes;

//module tree
mod visibility_attribute;
mod visibility_condition;
mod visibility_condition_constructors;

//API exports
pub use crate::visibility_attribute::*;
pub use crate::visibility_condition::*;
pub use crate::visibility_condition_constructors::*;

pub use bevy_replicon_attributes_derive::*;

pub mod prelude
{
    pub use crate::*;
}
