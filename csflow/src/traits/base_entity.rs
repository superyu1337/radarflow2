use memflow::types::Address;

use crate::{CheatCtx, structs::Vec3, Error};

/// A trait for basic functions from C_BaseEntity
/// CCSPlayerController inherits C_BaseEntity, which is why this trait exists.
pub trait BaseEntity {
    fn from_index(ctx: &mut CheatCtx, entity_list: Address, index: i32) -> Result<Option<Self>, Error> where Self: std::marker::Sized;
    fn pos(&self, ctx: &mut CheatCtx) -> Result<Vec3, Error>;
    fn class_name(&self, ctx: &mut CheatCtx) -> Result<String, Error>;
}