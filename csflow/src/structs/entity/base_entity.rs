use memflow::{types::Address, mem::MemoryView};
use crate::{CheatCtx, Error, cs2dumper, traits::{BaseEntity, MemoryClass}, structs::Vec3};

#[derive(Debug, Clone, Copy)]
pub struct CBaseEntity(Address);

impl MemoryClass for CBaseEntity {
    fn ptr(&self) -> Address {
        self.0
    }

    fn new(ptr: Address) -> Self {
        Self(ptr)
    }
}

impl BaseEntity for CBaseEntity {
    fn from_index(ctx: &mut CheatCtx, entity_list: Address, index: i32) -> Result<Option<CBaseEntity>, Error> {
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
        let entity_ptr = ctx.process.read_addr64(next_by_class_ptr)?;
        Ok(Self(entity_ptr))
    }

    fn previous_by_class(&self, ctx: &mut CheatCtx) -> Result<Self, Error> where Self: std::marker::Sized {
        let entity_identity_ptr = ctx.memory.read_addr64(self.0 + cs2dumper::client::CEntityInstance::m_pEntity)?;
        let next_by_class_ptr = ctx.process.read_addr64(entity_identity_ptr + cs2dumper::client::CEntityIdentity::m_pPrevByClass)?;
        let entity_ptr = ctx.process.read_addr64(next_by_class_ptr)?;
        Ok(Self(entity_ptr))
    }
}

impl CBaseEntity {
    pub fn to_player_controller(&self) -> super::CPlayerController {
        super::CPlayerController::new(self.0)
    }

    /// Professionally engineered function to quickly check if the entity has class name "weapon_c4"
    pub fn is_dropped_c4(&self, ctx: &mut CheatCtx) -> Result<bool, Error> {
        let entity_identity_ptr = ctx.process.read_addr64(self.0 + cs2dumper::client::CEntityInstance::m_pEntity)?;
        let class_name_ptr = ctx.process.read_addr64(entity_identity_ptr + cs2dumper::client::CEntityIdentity::m_designerName)?;

        let data = ctx.process.read_raw(class_name_ptr + 7, 2)?;
        let is_c4 = data == "c4".as_bytes();
        Ok(is_c4)
    }

    /// Professionally engineered function to quickly check if the entity has class name "cs_player_controller"
    pub fn is_cs_player_controller(&self, ctx: &mut CheatCtx) -> Result<bool, Error> {
        let entity_identity_ptr = ctx.process.read_addr64(self.0 + cs2dumper::client::CEntityInstance::m_pEntity)?;
        let class_name_ptr = ctx.process.read_addr64(entity_identity_ptr + cs2dumper::client::CEntityIdentity::m_designerName)?;

        let data = ctx.process.read_raw(class_name_ptr, 20)?;
        let is_controller = data == "cs_player_controller".as_bytes();
        Ok(is_controller)
    }
}