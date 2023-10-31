pub mod structs;
pub mod cs2dumper;
use crate::dma::CheatCtx;

use memflow::prelude::v1::*;
use anyhow::Result;

use self::structs::{CCSPlayerController, CPlayerPawn};

pub fn get_local(ctx: &mut CheatCtx) -> Result<CCSPlayerController> {
    let ptr = ctx.process.read_addr64(ctx.client_module.base + cs2dumper::offsets::client_dll::dwLocalPlayerController)?;
    Ok(CCSPlayerController::new(ptr))
}

pub fn get_local_pawn(ctx: &mut CheatCtx) -> Result<CPlayerPawn> {
    let ptr = ctx.process.read_addr64(ctx.client_module.base + cs2dumper::offsets::client_dll::dwLocalPlayerPawn)?;
    Ok(CPlayerPawn::new(ptr))
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

pub fn max_clients(global_vars: Address, ctx: &mut CheatCtx) -> Result<i32> {
    Ok(ctx.process.read(global_vars + 0x10)?)
}

pub fn is_ingame(ctx: &mut CheatCtx) -> Result<bool> {
    let ptr = ctx.process.read_addr64(ctx.engine_module.base + cs2dumper::offsets::engine2_dll::dwNetworkGameClient)?;
    let signonstate: u64 = ctx.process.read(ptr + cs2dumper::offsets::engine2_dll::dwNetworkGameClient_signOnState)?;
    Ok(signonstate == 6)
}