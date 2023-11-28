mod base_entity;
mod player_controller;
mod player_pawn;

pub use base_entity::CBaseEntity;
pub use player_controller::CPlayerController;
pub use player_pawn::CPlayerPawn;

use crate::{dma::CheatCtx, structs::Vec3};

use memflow::types::Address;
use anyhow::Result;

/// A trait that implements basic functions from C_BaseEntity
/// CCSPlayerController inherits C_BaseEntity, which is why this trait exists.
pub trait BaseEntity {
    fn from_index(ctx: &mut CheatCtx, entity_list: Address, index: i32) -> Result<Option<Self>> where Self: std::marker::Sized;
    fn pos(&self, ctx: &mut CheatCtx) -> Result<Vec3>;
    fn class_name(&self, ctx: &mut CheatCtx) -> Result<String>;
}

/// A trait that implements basic functions for an class represented by a single pointer
pub trait MemoryClass {
    fn ptr(&self) -> Address;
    fn new(ptr: Address) -> Self;
}