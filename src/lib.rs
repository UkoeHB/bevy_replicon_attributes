//documentation
#![doc = include_str!("../README.md")]
#![allow(unused_imports)]
use crate as bevy_replicon_attributes;

//module tree
mod temp;

//API exports
pub use crate::temp::*;

pub use bevy_replicon_attributes_derive::*;
