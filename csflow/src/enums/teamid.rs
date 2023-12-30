#[repr(i32)]
#[derive(Debug, Eq, PartialEq, enum_primitive_derive::Primitive)]
pub enum TeamID {
    Spectator = 1,
    T = 2,
    CT = 3
}