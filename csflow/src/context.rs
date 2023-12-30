use memflow::{plugins::{IntoProcessInstanceArcBox, Inventory, ConnectorArgs, args::Args}, os::{ModuleInfo, Os, Process}, mem::MemoryView, types::Address};

use crate::{error::Error, structs::{CPlayerController, CBaseEntity}, cs2dumper, traits::MemoryClass};

pub struct CheatCtx {
    pub process: IntoProcessInstanceArcBox<'static>,
    pub client_module: ModuleInfo,
    pub engine_module: ModuleInfo,
}

impl CheatCtx {
    fn check_version(&mut self) -> Result<(), Error> {
        let game_build_number: u32 = self.process.read(self.engine_module.base + cs2dumper::offsets::engine2_dll::dwBuildNumber)?;
        let offset_build_number = cs2dumper::offsets::game_info::buildNumber;

        if game_build_number as usize != offset_build_number {
            return Err(Error::GameVersionMismatch(game_build_number as usize, offset_build_number))
        }

        Ok(())
    }

    pub fn setup(connector: Connector, pcileech_device: String) -> Result<CheatCtx, Error> {
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

    pub fn get_local(&mut self) -> Result<CPlayerController, Error> {
        let ptr = self.process.read_addr64(self.client_module.base + cs2dumper::offsets::client_dll::dwLocalPlayerController)?;
        Ok(CPlayerController::new(ptr))
    }
    
    pub fn get_plantedc4(&mut self) -> Result<CBaseEntity, Error> {
        let ptr = self.process.read_addr64(self.client_module.base + cs2dumper::offsets::client_dll::dwPlantedC4)?;
        let ptr2 = self.process.read_addr64(ptr)?;
        Ok(CBaseEntity::new(ptr2))
    }
    
    pub fn is_bomb_planted(&mut self) -> Result<bool, Error> {
        let game_rules = self.process.read_addr64(self.client_module.base + cs2dumper::offsets::client_dll::dwGameRules)?;
        let data: u8 = self.process.read(game_rules + cs2dumper::client::C_CSGameRules::m_bBombPlanted)?;
        Ok(data != 0)
    }
    
    pub fn is_bomb_dropped(&mut self) -> Result<bool, Error> {
        let game_rules = self.process.read_addr64(self.client_module.base + cs2dumper::offsets::client_dll::dwGameRules)?;
        let data: u8 = self.process.read(game_rules + cs2dumper::client::C_CSGameRules::m_bBombDropped)?;
        Ok(data != 0)
    }

    pub fn get_entity_list(&mut self) -> Result<Address, Error> {
        let ptr = self.process.read_addr64(self.client_module.base + cs2dumper::offsets::client_dll::dwEntityList)?;
        Ok(ptr)
    }
    
    pub fn get_globals(&mut self) -> Result<Address, Error> {
        let ptr = self.process.read_addr64(self.client_module.base + cs2dumper::offsets::client_dll::dwGlobalVars)?;
        Ok(ptr)
    }
    
    pub fn map_name(&mut self, global_vars: Address) -> Result<String, Error> {
        let ptr = self.process.read_addr64(global_vars + 0x188)?;
        Ok(self.process.read_char_string_n(ptr, 32)?)
    }
    
    pub fn highest_entity_index(&mut self) -> Result<i32, Error> {
        let game_entity_system = self.process.read_addr64(self.client_module.base + cs2dumper::offsets::client_dll::dwGameEntitySystem)?;
        let highest_index = self.process.read(game_entity_system + cs2dumper::offsets::client_dll::dwGameEntitySystem_getHighestEntityIndex)?;
        Ok(highest_index)
    }
    
    pub fn network_is_ingame(&mut self) -> Result<bool, Error> {
        let ptr = self.process.read_addr64(self.engine_module.base + cs2dumper::offsets::engine2_dll::dwNetworkGameClient)?;
        let signonstate: i32 = self.process.read(ptr + cs2dumper::offsets::engine2_dll::dwNetworkGameClient_signOnState)?;
        Ok(signonstate == 6)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Default)]
pub enum Connector {
    #[default]
    Qemu,
    Kvm,
    Pcileech
}

impl ToString for Connector {
    fn to_string(&self) -> String {
        match self {
            Connector::Qemu => String::from("qemu"),
            Connector::Kvm => String::from("kvm"),
            Connector::Pcileech => String::from("pcileech"),
        }
    }
}