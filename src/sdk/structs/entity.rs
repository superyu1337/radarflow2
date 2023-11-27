use crate::{dma::CheatCtx, sdk::cs2dumper, structs::{Vec3, communication::PlayerType}};
use enum_primitive_derive::Primitive;
use memflow::{prelude::MemoryView, types::Address};
use anyhow::Result;
use num_traits::FromPrimitive;

pub trait MemoryClass {
    fn ptr(&self) -> Address;
    fn new(ptr: Address) -> Self;
}

#[repr(i32)]
#[derive(Debug, Eq, PartialEq, Primitive)]
pub enum TeamID {
    Spectator = 1,
    T = 2,
    CT = 3
}

pub struct PlayerController(Address);

impl PlayerController {

    /*
    pub fn from_entity_list(ctx: &mut CheatCtx, entity_list: Address, index: i32) -> Result<Option<Self>> {
        let list_entry = ctx.process.read_addr64(entity_list + ((8 * (index & 0x7FFF)) >> 9) + 16)?;
        if list_entry.is_null() && !list_entry.is_valid() {
            return Ok(None);
        }

        let player_ptr = ctx.process.read_addr64(list_entry + 120 * (index & 0x1FF))?;
        if player_ptr.is_null() && !player_ptr.is_valid() {
            return Ok(None);
        }

        Ok(Some(Self::new(player_ptr)))
    }
    */

    pub fn from_entity_list_v2(ctx: &mut CheatCtx, entity_list: Address, index: i32) -> Result<Option<Self>> {
        let list_entry = ctx.process.read_addr64(entity_list + 8 * (index >> 9) + 16)?;
        if list_entry.is_null() && !list_entry.is_valid() {
            return Ok(None);
        }

        let player_ptr = ctx.process.read_addr64(list_entry + 120 * (index & 0x1FF))?;
        if player_ptr.is_null() && !player_ptr.is_valid() {
            return Ok(None);
        }

        Ok(Some(Self::new(player_ptr)))
    }

    pub fn pos(&self, ctx: &mut CheatCtx) -> Result<Vec3> {
        let node = ctx.process.read_addr64(self.0 + cs2dumper::client::C_BaseEntity::m_pGameSceneNode)?;
        Ok(ctx.process.read(node + cs2dumper::client::CGameSceneNode::m_vecAbsOrigin)?)
    }

    pub fn class_name(&self, ctx: &mut CheatCtx) -> Result<String> {
        let entity_identity_ptr = ctx.process.read_addr64(self.0 + cs2dumper::client::CEntityInstance::m_pEntity)?;
        let class_name_ptr = ctx.process.read_addr64(entity_identity_ptr + cs2dumper::client::CEntityIdentity::m_designerName)?;
        Ok(ctx.process.read_char_string_n(class_name_ptr, 32)?)
    }

    pub fn get_team(&self, ctx: &mut CheatCtx) -> Result<Option<TeamID>> {
        let team_num: i32 = ctx.process.read(self.0 + cs2dumper::client::C_BaseEntity::m_iTeamNum)?;
        Ok(TeamID::from_i32(team_num))
    }

    pub fn get_player_type(&self, ctx: &mut CheatCtx, local: &PlayerController) -> Result<Option<PlayerType>> {
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

    pub fn pawn(&self, ctx: &mut CheatCtx, entity_list: Address) -> Result<Option<PlayerPawn>> {
        let uhandle = ctx.process.read(self.0 + cs2dumper::client::CCSPlayerController::m_hPlayerPawn)?;
        PlayerPawn::from_uhandle(ctx, entity_list, uhandle)
    }

}

impl MemoryClass for PlayerController {
    fn ptr(&self) -> Address {
        self.0
    }

    fn new(ptr: Address) -> Self {
        Self(ptr)
    }
}

pub struct PlayerPawn(Address);

impl PlayerPawn {
    pub fn from_uhandle(ctx: &mut CheatCtx, entity_list: Address, uhandle: u32) -> Result<Option<Self>> {
        let list_entry = ctx.process.read_addr64(entity_list + 0x8 * ((uhandle & 0x7FFF) >> 9) + 16)?;
        
        if list_entry.is_null() || !list_entry.is_valid() {
            Ok(None)
        } else {
            let ptr = ctx.process.read_addr64(list_entry + 120 * (uhandle & 0x1FF))?;
            Ok(Some(Self(ptr)))
        }
    }

    pub fn has_c4(&self, ctx: &mut CheatCtx, entity_list: Address) -> Result<bool> {
        let mut has_c4 = false;
        let wep_services = ctx.process.read_addr64(self.0 + cs2dumper::client::C_BasePlayerPawn::m_pWeaponServices)?;
        let wep_count: i32  = ctx.process.read(wep_services + cs2dumper::client::CPlayer_WeaponServices::m_hMyWeapons)?;
        let wep_base = ctx.process.read_addr64(wep_services + cs2dumper::client::CPlayer_WeaponServices::m_hMyWeapons + 0x8)?;

        for wep_idx in 0..wep_count {
            let handle: i32 = ctx.process.read(wep_base + wep_idx * 0x4)?;
            if handle == -1 {
                continue;
            }

            let list_entry = ctx.process.read_addr64(entity_list + 0x8 * ((handle & 0x7FFF) >> 9) + 16)?;
            if let Some(wep_ptr) = {
                if list_entry.is_null() || !list_entry.is_valid() {
                    None
                } else {
                    let ptr = ctx.process.read_addr64(list_entry + 120 * (handle & 0x1FF))?;
                    Some(ptr)
                }
            } {
                let wep_data = ctx.process.read_addr64(wep_ptr + cs2dumper::client::C_BaseEntity::m_nSubclassID + 0x8)?;
                let id: i32 = ctx.process.read(wep_data + cs2dumper::client::CCSWeaponBaseVData::m_WeaponType)?;

                if id == 7 {
                    has_c4 = true;
                    break;
                }
            }
        }

        Ok(has_c4)
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

impl MemoryClass for PlayerPawn {
    fn ptr(&self) -> Address {
        self.0
    }

    fn new(ptr: Address) -> Self {
        Self(ptr)
    }
}

pub struct Bomb(Address);

impl MemoryClass for Bomb {
    fn ptr(&self) -> Address {
        self.0
    }

    fn new(ptr: Address) -> Self {
        Self(ptr)
    }
}

impl Bomb {
    pub fn pos(&self, ctx: &mut CheatCtx) -> Result<Vec3> {
        let c4_node = ctx.process.read_addr64(self.0 + cs2dumper::client::C_BaseEntity::m_pGameSceneNode)?;
        Ok(ctx.process.read(c4_node + cs2dumper::client::CGameSceneNode::m_vecAbsOrigin)?)
    }
}