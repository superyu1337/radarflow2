pub mod structs;
pub mod cs2dumper;
use crate::dma::CheatCtx;

use memflow::prelude::v1::*;
use anyhow::Result;

use self::structs::{PlayerController, PlayerPawn, MemoryClass, Bomb};

pub fn get_local(ctx: &mut CheatCtx) -> Result<PlayerController> {
    let ptr = ctx.process.read_addr64(ctx.client_module.base + cs2dumper::offsets::client_dll::dwLocalPlayerController)?;
    Ok(PlayerController::new(ptr))
}

pub fn get_plantedc4(ctx: &mut CheatCtx) -> Result<Bomb> {
    let ptr = ctx.process.read_addr64(ctx.client_module.base + cs2dumper::offsets::client_dll::dwPlantedC4)?;
    let ptr2 = ctx.process.read_addr64(ptr)?;
    Ok(Bomb::new(ptr2))
}

pub fn is_bomb_planted(ctx: &mut CheatCtx) -> Result<bool> {
    let game_rules = ctx.process.read_addr64(ctx.client_module.base + cs2dumper::offsets::client_dll::dwGameRules)?;
    let data: u8 = ctx.process.read(game_rules + cs2dumper::client::C_CSGameRules::m_bBombPlanted)?;
    Ok(data != 0)
}

pub fn is_bomb_dropped(ctx: &mut CheatCtx) -> Result<bool> {
    let game_rules = ctx.process.read_addr64(ctx.client_module.base + cs2dumper::offsets::client_dll::dwGameRules)?;
    let data: u8 = ctx.process.read(game_rules + cs2dumper::client::C_CSGameRules::m_bBombDropped)?;
    Ok(data != 0)
}

#[allow(dead_code)]
pub fn get_local_pawn(ctx: &mut CheatCtx) -> Result<PlayerPawn> {
    let ptr = ctx.process.read_addr64(ctx.client_module.base + cs2dumper::offsets::client_dll::dwLocalPlayerPawn)?;
    Ok(PlayerPawn::new(ptr))
}

pub fn get_entity_list(ctx: &mut CheatCtx) -> Result<Address> {
    let ptr = ctx.process.read_addr64(ctx.client_module.base + cs2dumper::offsets::client_dll::dwEntityList)?;
    Ok(ptr)
}

pub fn get_globals(ctx: &mut CheatCtx) -> Result<Address> {
    let ptr = ctx.process.read_addr64(ctx.client_module.base + cs2dumper::offsets::client_dll::dwGlobalVars)?;
    Ok(ptr)
}

pub fn map_name(global_vars: Address, ctx: &mut CheatCtx) -> Result<String> {
    let ptr = ctx.process.read_addr64(global_vars + 0x188)?;
    Ok(ctx.process.read_char_string_n(ptr, 32)?)
}

/* 
pub fn max_clients(global_vars: Address, ctx: &mut CheatCtx) -> Result<i32> {
    Ok(ctx.process.read(global_vars + 0x10)?)
}
*/

pub fn highest_entity_index(ctx: &mut CheatCtx) -> Result<i32> {
    let game_entity_system = ctx.process.read_addr64(ctx.client_module.base + cs2dumper::offsets::client_dll::dwGameEntitySystem)?;
    let highest_index = ctx.process.read(game_entity_system + cs2dumper::offsets::client_dll::dwGameEntitySystem_getHighestEntityIndex)?;
    Ok(highest_index)
}

pub fn is_ingame(ctx: &mut CheatCtx) -> Result<bool> {
    let ptr = ctx.process.read_addr64(ctx.engine_module.base + cs2dumper::offsets::engine2_dll::dwNetworkGameClient)?;
    let signonstate: u64 = ctx.process.read(ptr + cs2dumper::offsets::engine2_dll::dwNetworkGameClient_signOnState)?;
    Ok(signonstate == 6)
}