use memflow::{types::Address, mem::MemoryView};
use num_traits::FromPrimitive;

use crate::{CheatCtx, Error, cs2dumper, structs::Vec3, traits::{MemoryClass, BaseEntity}, enums::{TeamID, PlayerType}};

#[derive(Debug, Clone, Copy)]
pub struct CPlayerController(Address);

impl MemoryClass for CPlayerController {
    fn ptr(&self) -> Address {
        self.0
    }

    fn new(ptr: Address) -> Self {
        Self(ptr)
    }
}

impl BaseEntity for CPlayerController {
    fn from_index(ctx: &mut CheatCtx, entity_list: Address, index: i32) -> Result<Option<CPlayerController>, Error> {
        let list_entry = ctx.memory.read_addr64(entity_list + 8 * (index >> 9) + 16)?;
        if list_entry.is_null() && !list_entry.is_valid() {
            return Ok(None);
        }

        let player_ptr = ctx.memory.read_addr64(list_entry + 120 * (index & 0x1FF))?;
        if player_ptr.is_null() && !player_ptr.is_valid() {
            return Ok(None);
        }

        Ok(Some(Self::new(player_ptr)))
    }

    /// This function is slower because of an additional pointer read, use pawn.pos() instead.
    fn pos(&self, ctx: &mut CheatCtx) -> Result<Vec3, Error> {
        let node = ctx.memory.read_addr64(self.0 + cs2dumper::client::C_BaseEntity::m_pGameSceneNode)?;
        Ok(ctx.memory.read(node + cs2dumper::client::CGameSceneNode::m_vecAbsOrigin)?)
    }

    fn class_name(&self, ctx: &mut CheatCtx) -> Result<String, Error> {
        let entity_identity_ptr = ctx.process.read_addr64(self.0 + cs2dumper::client::CEntityInstance::m_pEntity)?;
        let class_name_ptr = ctx.process.read_addr64(entity_identity_ptr + cs2dumper::client::CEntityIdentity::m_designerName)?;
        Ok(ctx.memory.read_char_string_n(class_name_ptr, 32)?)
    }

    fn next_by_class(&self, ctx: &mut CheatCtx) -> Result<Self, Error> where Self: std::marker::Sized {
        let entity_identity_ptr = ctx.process.read_addr64(self.0 + cs2dumper::client::CEntityInstance::m_pEntity)?;
        let next_by_class_ptr = ctx.process.read_addr64(entity_identity_ptr + cs2dumper::client::CEntityIdentity::m_pNextByClass)?;

        let class_name_ptr = ctx.process.read_addr64(entity_identity_ptr + cs2dumper::client::CEntityIdentity::m_designerName)?;
        let class_name= ctx.process.read_char_string_n(class_name_ptr, 32)?;

        println!("class_name: {}", class_name);

        let stringable_index: i32 = ctx.process.read(entity_identity_ptr + cs2dumper::client::CEntityIdentity::m_nameStringableIndex)?;
        println!("stringable_index: {}", stringable_index);

        let entity_ptr = ctx.process.read_addr64(next_by_class_ptr)?;
        println!("entity_ptr: {}", entity_ptr);
        Ok(Self(entity_ptr))
    }

    fn previous_by_class(&self, ctx: &mut CheatCtx) -> Result<Self, Error> where Self: std::marker::Sized {
        let entity_identity_ptr = ctx.process.read_addr64(self.0 + cs2dumper::client::CEntityInstance::m_pEntity)?;
        let next_by_class_ptr = ctx.process.read_addr64(entity_identity_ptr + cs2dumper::client::CEntityIdentity::m_pPrevByClass)?;
        let entity_ptr = ctx.process.read_addr64(next_by_class_ptr)?;
        Ok(Self(entity_ptr))
    }
}

impl CPlayerController {
    pub fn get_pawn(&self, ctx: &mut CheatCtx, entity_list: Address) -> Result<Option<super::CPlayerPawn>, Error> {
        let uhandle = ctx.memory.read(self.0 + cs2dumper::client::CCSPlayerController::m_hPlayerPawn)?;
        super::CPlayerPawn::from_uhandle(ctx, entity_list, uhandle)
    }

    // Technically something that should be in the BaseEntity trait, but we are only gonna use it on CPlayerController
    pub fn get_team(&self, ctx: &mut CheatCtx) -> Result<Option<TeamID>, Error> {
        let team_num: i32 = ctx.memory.read(self.0 + cs2dumper::client::C_BaseEntity::m_iTeamNum)?;
        Ok(TeamID::from_i32(team_num))
    }

    pub fn get_player_type(&self, ctx: &mut CheatCtx, local: &CPlayerController) -> Result<Option<PlayerType>, Error> {
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
}