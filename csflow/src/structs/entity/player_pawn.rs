use memflow::{types::Address, mem::MemoryView};

use crate::{Error, CheatCtx, cs2dumper, structs::Vec3, traits::MemoryClass};

#[derive(Debug, Clone, Copy)]
pub struct CPlayerPawn(Address);

impl MemoryClass for CPlayerPawn {
    fn ptr(&self) -> Address {
        self.0
    }

    fn new(ptr: Address) -> Self {
        Self(ptr)
    }
}

impl CPlayerPawn {
    pub fn from_uhandle(ctx: &mut CheatCtx, entity_list: Address, uhandle: u32) -> Result<Option<Self>, Error> {
        let list_entry = ctx.process.read_addr64(entity_list + 0x8 * ((uhandle & 0x7FFF) >> 9) + 16)?;
        
        if list_entry.is_null() || !list_entry.is_valid() {
            Ok(None)
        } else {
            let ptr = ctx.process.read_addr64(list_entry + 120 * (uhandle & 0x1FF))?;
            Ok(Some(Self(ptr)))
        }
    }

    // Todo: Optimize this function: find another way to do this
    pub fn has_c4(&self, ctx: &mut CheatCtx, entity_list: Address) -> Result<bool, Error> {
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

    pub fn pos(&self, ctx: &mut CheatCtx) -> Result<Vec3, Error> {
        Ok(ctx.process.read(self.0 + cs2dumper::client::C_BasePlayerPawn::m_vOldOrigin)?)
    }

    pub fn angles(&self, ctx: &mut CheatCtx) -> Result<Vec3, Error> {
        Ok(ctx.process.read(self.0 + cs2dumper::client::C_CSPlayerPawnBase::m_angEyeAngles)?)
    }

    pub fn health(&self, ctx: &mut CheatCtx) -> Result<u32, Error> {
        Ok(ctx.process.read(self.0 + cs2dumper::client::C_BaseEntity::m_iHealth)?)
    }

    /// Same as ::get_health > 0
    pub fn is_alive(&self, ctx: &mut CheatCtx) -> Result<bool, Error> {
        Ok(self.health(ctx)? > 0)
    }
}