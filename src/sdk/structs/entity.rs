use crate::{dma::CheatCtx, sdk::cs2dumper, structs::{Vec3, communication::PlayerType}};
use enum_primitive_derive::Primitive;
use memflow::{prelude::MemoryView, types::Address};
use anyhow::{Result, anyhow};
use num_traits::FromPrimitive;

#[repr(i32)]
#[derive(Debug, Eq, PartialEq, Primitive)]
pub enum TeamID {
    Spectator = 1,
    T = 2,
    CT = 3
}

pub struct CEntityIdentity(Address);

impl CEntityIdentity {
    pub fn prev_by_class(&self, ctx: &mut CheatCtx) -> Result<CBaseEntity> {
        let prev1 = ctx.process.read_addr64(self.0 + cs2dumper::client::CEntityIdentity::m_pPrevByClass)?;
        let prev = ctx.process.read_addr64(prev1)?;
 
        if prev.is_null() || !prev.is_valid() {
            Err(anyhow!("Invalid or Null"))
        } else {
            Ok(CBaseEntity(prev))
        }
    }

    pub fn next_by_class(&self, ctx: &mut CheatCtx) -> Result<CBaseEntity> {
        let next1 = ctx.process.read_addr64(self.0 + cs2dumper::client::CEntityIdentity::m_pNextByClass)?;
        let next = ctx.process.read_addr64(next1)?;
                
        if next.is_null() || !next.is_valid() {
            Err(anyhow!("Invalid or Null"))
        } else {
            Ok(CBaseEntity(next))
        }
    }

    pub fn designer_name(&self, ctx: &mut CheatCtx) -> Result<String> {
        let ptr = ctx.process.read_addr64(self.0 + cs2dumper::client::CEntityIdentity::m_designerName)?;
        Ok(ctx.process.read_char_string_n(ptr, 32)?)
    }
}

pub struct CBaseEntity(Address);

impl CBaseEntity {
    pub fn new(ptr: Address) -> CBaseEntity {
        CBaseEntity(ptr)
    }

    pub fn ptr(&self) -> Address {
        self.0
    }

    pub fn to_controller(&self) -> CCSPlayerController {
        CCSPlayerController(self.0)
    }

    pub fn get_from_list(ctx: &mut CheatCtx, entity_list: Address, idx: usize) -> Result<CBaseEntity> {
        let list_entry = ctx.process.read_addr64(entity_list + ((( idx & 0x7FFF ) >> 9 ) * 0x8)).unwrap();

        if list_entry.is_null() || !list_entry.is_valid() {
            Err(anyhow!("Invalid or Null"))
        } else {
            let ptr = ctx.process.read_addr64(list_entry + 120 * (idx & 0x1FF)).unwrap();
            Ok(CBaseEntity(ptr))
        }
    }

    pub fn entity_identity(&self, ctx: &mut CheatCtx) -> Result<CEntityIdentity> {
        let ptr = ctx.process.read_addr64(self.0 + cs2dumper::client::CEntityInstance::m_pEntity)?;
        Ok(CEntityIdentity(ptr))
    }
}

pub struct CPlayerPawn(Address);

impl CPlayerPawn {
    pub fn new(ptr: Address) -> CPlayerPawn {
        CPlayerPawn(ptr)
    }
    
    pub fn from_uhandle(uhandle: u32, entity_list: Address, ctx: &mut CheatCtx) -> Option<CPlayerPawn> {
        let list_entry = ctx.process.read_addr64(entity_list + 0x8 * ((uhandle & 0x7FFF) >> 9) + 16).unwrap();
        
        if list_entry.is_null() || !list_entry.is_valid() {
            None
        } else {
            let ptr = ctx.process.read_addr64(list_entry + 120 * (uhandle & 0x1FF)).unwrap();
            Some(CPlayerPawn(ptr))
        }
    }

    /*
        DWORD64 entityPawnBase = Memory::Read<unsigned __int64>(EntitiesList + ((hEntity & 0x7FFF)  * ENTITY_SPACING));
        auto pawn = read<C_CSPlayerPawnBase*>(entityPawnBase + 0x78 * (hEntity & 0x1FF));
    */

    pub fn from_uhandle2(uhandle: u32, entity_list: Address, ctx: &mut CheatCtx) -> Option<CPlayerPawn> {
        let ent_pawn_base = ctx.process.read_addr64(entity_list + (uhandle & 0x7FFF) * 0x10).unwrap();
        
        if ent_pawn_base.is_null() || !ent_pawn_base.is_valid() {
            None
        } else {
            let ptr = ctx.process.read_addr64(ent_pawn_base + 0x78 * (uhandle & 0x1FF)).unwrap();
            Some(CPlayerPawn(ptr))
        }
    }

    pub fn ptr(&self) -> Address {
        self.0
    }

    pub fn pos(&self, ctx: &mut CheatCtx) -> Result<Vec3> {
        Ok(ctx.process.read(self.0 + cs2dumper::client::C_BasePlayerPawn::m_vOldOrigin)?)
    }

    pub fn angles(&self, ctx: &mut CheatCtx) -> Result<Vec3> {
        Ok(ctx.process.read(self.0 + cs2dumper::client::C_CSPlayerPawnBase::m_angEyeAngles)?)
    }

    pub fn health(&self, ctx: &mut CheatCtx) -> Result<u32> {
        Ok(ctx.process.read(self.0 + cs2dumper::client::C_BaseEntity::m_iHealth)?)
    }

    /// Same as ::get_health > 0
    pub fn is_alive(&self, ctx: &mut CheatCtx) -> Result<bool> {
        Ok(self.health(ctx)? > 0)
    }

}

pub struct CCSPlayerController(Address);

impl CCSPlayerController {
    pub fn ptr(&self) -> Address {
        self.0
    }

    pub fn new(ptr: Address) -> CCSPlayerController {
        CCSPlayerController(ptr)
    }

    pub fn get_team(&self, ctx: &mut CheatCtx) -> Result<Option<TeamID>> {
        let team: i32 = ctx.process.read(self.0 + cs2dumper::client::C_BaseEntity::m_iTeamNum)?;
        Ok(TeamID::from_i32(team))
    }

    pub fn get_player_type(&self, ctx: &mut CheatCtx, local: &CCSPlayerController) -> Result<Option<PlayerType>> {
        if self.0 == local.0 {
            return Ok(Some(PlayerType::Local))
        }

        let team = {
            match self.get_team(ctx)? {
                Some(t) => t,
                None => { return Ok(None) },
            }
        };
        
        let local_team = {
            match local.get_team(ctx)? {
                Some(t) => t,
                None => { return Ok(None) },
            }
        };

        let player_type = {
            if team == TeamID::Spectator { 
                PlayerType::Spectator 
            } else if team != local_team {
                PlayerType::Enemy
            } else {
                PlayerType::Team
            }
        };

        Ok(Some(player_type))
    }

    pub fn pawn(&self, entity_list: Address, ctx: &mut CheatCtx) -> Result<Option<CPlayerPawn>> {
        let uhandle = ctx.process.read(self.0 + cs2dumper::client::CCSPlayerController::m_hPlayerPawn)?;
        Ok(CPlayerPawn::from_uhandle(uhandle, entity_list, ctx))
    }

    pub fn pawn2(&self, entity_list: Address, ctx: &mut CheatCtx) -> Result<Option<CPlayerPawn>> {
        let uhandle = ctx.process.read(self.0 + cs2dumper::client::CBasePlayerController::m_hPawn)?;
        Ok(CPlayerPawn::from_uhandle2(uhandle, entity_list, ctx))
    }

    pub fn player_name(&self, ctx: &mut CheatCtx) -> Result<String> {
        let ptr = ctx.process.read_addr64(self.0 + cs2dumper::client::CCSPlayerController::m_sSanitizedPlayerName)?;
        Ok(ctx.process.read_char_string_n(ptr, 32)?)
    }

    pub fn entity_identity(&self, ctx: &mut CheatCtx) -> Result<CEntityIdentity> {
        let ptr = ctx.process.read_addr64(self.0 + cs2dumper::client::CEntityInstance::m_pEntity)?;
        Ok(CEntityIdentity(ptr))
    }

    pub fn to_base(&self) -> CBaseEntity {
        CBaseEntity(self.0)
    }
}