use memflow::prelude::v1::*;

mod connector;

pub use connector::Connector;
use num_traits::FromPrimitive;

use crate::{structs::Vec3, enums::TeamID};

use super::cs2dumper;

pub struct DmaCtx {
    pub process: IntoProcessInstanceArcBox<'static>,
    pub client_module: ModuleInfo,
    pub engine_module: ModuleInfo,
}

impl DmaCtx {
    fn check_version(&mut self) -> anyhow::Result<()> {
        let game_build_number: u32 = self.process.read(self.engine_module.base + cs2dumper::offsets::engine2_dll::dwBuildNumber)?;
        let offset_build_number = env!("CS2_BUILD_NUMBER").parse::<usize>()?;

        if game_build_number as usize != offset_build_number {
            return Err(anyhow::anyhow!(
                "game build is {}, but offsets are for {}",
                game_build_number,
                offset_build_number
            ));
        }

        Ok(())
    }

    pub fn setup(connector: Connector, pcileech_device: String) -> anyhow::Result<DmaCtx> {
        let inventory = Inventory::scan();

        let os = { 
            if connector == Connector::Pcileech {
                let args = Args::new()
                    .insert("device", &pcileech_device);

                let connector_args = ConnectorArgs::new(None, args, None);                

                inventory.builder()
                    .connector(&connector.to_string())
                    .args(connector_args)
                    .os("win32")
                    .build()?
            } else {
                inventory.builder()
                .connector(&connector.to_string())
                .os("win32")
                .build()?
            }
        };

        let mut process = os.into_process_by_name("cs2.exe")?;

        let client_module = process.module_by_name("client.dll")?;

        let engine_module = process.module_by_name("engine2.dll")?;

        let mut ctx = Self {
            process,
            client_module,
            engine_module,
        };

        ctx.check_version()?;

        Ok(ctx)
    }

    pub fn pawn_from_controller(&mut self, controller: Address, entity_list: Address) -> anyhow::Result<Option<Address>> {
        let uhandle: u32 = self.process.read(controller + cs2dumper::client::CCSPlayerController::m_hPlayerPawn)?;

        let list_entry = self.process.read_addr64(entity_list + 0x8 * ((uhandle & 0x7FFF) >> 9) + 16)?;
        
        if list_entry.is_null() || !list_entry.is_valid() {
            Ok(None)
        } else {
            let ptr = self.process.read_addr64(list_entry + 120 * (uhandle & 0x1FF))?;
            Ok(Some(ptr))
        }

        //super::CPlayerPawn::from_uhandle(ctx, entity_list, uhandle)
    }

    pub fn batched_player_read(&mut self, controller: Address, pawn: Address) -> anyhow::Result<BatchedPlayerData> {
        let mut pos = Vec3::default();
        let mut yaw = 0f32;
        let mut health = 0u32;
        let mut team = 0i32;
        let mut clipping_weapon = 0u64;
        let mut is_scoped = 0u8;

        {
            let mut batcher = MemoryViewBatcher::new(&mut self.process);
            batcher.read_into(pawn + cs2dumper::client::C_BasePlayerPawn::m_vOldOrigin, &mut pos);
            batcher.read_into(pawn + cs2dumper::client::C_CSPlayerPawnBase::m_angEyeAngles + 4, &mut yaw);
            batcher.read_into(pawn + cs2dumper::client::C_BaseEntity::m_iHealth, &mut health);
            batcher.read_into(controller + cs2dumper::client::C_BaseEntity::m_iTeamNum, &mut team);
            batcher.read_into(pawn + cs2dumper::client::C_CSPlayerPawnBase::m_pClippingWeapon, &mut clipping_weapon);
            batcher.read_into(pawn + cs2dumper::client::C_CSPlayerPawnBase::m_bIsScoped, &mut is_scoped);
        }
    
        let team = TeamID::from_i32(team);

        let has_awp = {
            let clipping_weapon: Address = clipping_weapon.into();
            let items_def_idx_addr = clipping_weapon + cs2dumper::client::C_EconEntity::m_AttributeManager 
                + cs2dumper::client::C_AttributeContainer::m_Item + cs2dumper::client::C_EconItemView::m_iItemDefinitionIndex;
    
            let items_def_idx: i16 = self.process.read(items_def_idx_addr)?;

            items_def_idx == 9
        };

        Ok(BatchedPlayerData {
            pos,
            yaw,
            team,
            health,
            has_awp,
            is_scoped: is_scoped != 0
        })
    }

    pub fn get_plantedc4(&mut self) -> anyhow::Result<Address> {
        let ptr = self.process.read_addr64(self.client_module.base + cs2dumper::offsets::client_dll::dwPlantedC4)?;
        let ptr2 = self.process.read_addr64(ptr)?;
        Ok(ptr2)
    }

    /// Professionally engineered function to quickly check if the entity has class name "weapon_c4"
    pub fn is_dropped_c4(&mut self, entity_ptr: Address) -> anyhow::Result<bool> {
        let entity_identity_ptr = self.process.read_addr64(entity_ptr + cs2dumper::client::CEntityInstance::m_pEntity)?;
        let class_name_ptr = self.process.read_addr64(entity_identity_ptr + cs2dumper::client::CEntityIdentity::m_designerName)?;

        let data = self.process.read_raw(class_name_ptr + 7, 2)?;
        let is_c4 = data == "c4".as_bytes();
        Ok(is_c4)
    }

    /// Professionally engineered function to quickly check if the entity has class name "cs_player_controller"
    pub fn is_cs_player_controller(&mut self, entity_ptr: Address) -> anyhow::Result<bool> {
        let entity_identity_ptr = self.process.read_addr64(entity_ptr + cs2dumper::client::CEntityInstance::m_pEntity)?;
        let class_name_ptr = self.process.read_addr64(entity_identity_ptr + cs2dumper::client::CEntityIdentity::m_designerName)?;

        let data = self.process.read_raw(class_name_ptr, 20)?;
        let is_controller = data == "cs_player_controller".as_bytes();
        Ok(is_controller)
    }

    // Todo: Optimize this function: find another way to do this
    pub fn has_c4(&mut self, pawn: Address, entity_list: Address) -> anyhow::Result<bool> {
        let mut has_c4 = false;
        let wep_services = self.process.read_addr64(pawn + cs2dumper::client::C_BasePlayerPawn::m_pWeaponServices)?;
        let wep_count: i32  = self.process.read(wep_services + cs2dumper::client::CPlayer_WeaponServices::m_hMyWeapons)?;
        let wep_base = self.process.read_addr64(wep_services + cs2dumper::client::CPlayer_WeaponServices::m_hMyWeapons + 0x8)?;

        for wep_idx in 0..wep_count {
            let handle: i32 = self.process.read(wep_base + wep_idx * 0x4)?;
            if handle == -1 {
                continue;
            }

            let list_entry = self.process.read_addr64(entity_list + 0x8 * ((handle & 0x7FFF) >> 9) + 16)?;
            if let Some(wep_ptr) = {
                if list_entry.is_null() || !list_entry.is_valid() {
                    None
                } else {
                    let ptr = self.process.read_addr64(list_entry + 120 * (handle & 0x1FF))?;
                    Some(ptr)
                }
            } {
                let wep_data = self.process.read_addr64(wep_ptr + cs2dumper::client::C_BaseEntity::m_nSubclassID + 0x8)?;
                let id: i32 = self.process.read(wep_data + cs2dumper::client::CCSWeaponBaseVData::m_WeaponType)?;

                if id == 7 {
                    has_c4 = true;
                    break;
                }
            }
        }

        Ok(has_c4)
    }

}


#[derive(Debug)]
pub struct BatchedPlayerData {
    pub pos: Vec3,
    pub yaw: f32,
    pub team: Option<TeamID>,
    pub health: u32,
    pub has_awp: bool,
    pub is_scoped: bool,
}