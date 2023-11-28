mod entity_classes;

pub use entity_classes::*;

use enum_primitive_derive::Primitive;

#[repr(i32)]
#[derive(Debug, Eq, PartialEq, Primitive)]
pub enum TeamID {
    Spectator = 1,
    T = 2,
    CT = 3
}