use itertools::Itertools;
use memflow::{mem::MemoryView, types::Address};
use tokio::time::Instant;

use super::{context::DmaCtx, cs2dumper};

#[derive(Clone, Debug, Default)]
pub struct CsData {
    // Entities
    pub players: Vec<(Address, Address)>,
    pub bomb: Address,
    pub bomb_holder: Option<Address>,
    pub recheck_bomb_holder: bool,

    // Pointers
    pub globals: u64,
    pub gamerules: u64,
    pub entity_list: u64,
    pub game_ent_sys: u64,

    // Common
    pub local: u64,
    pub local_pawn: u64,
    // pub is_dead: bool,   // TODO: Why is this here?
    pub tick_count: i32,
    pub freeze_period: bool,
    pub round_start_count: u8,
    pub highest_index: i32,
    pub map: String,

    // Bomb
    pub bomb_dropped: bool,
    pub bomb_planted: bool,
    pub bomb_planted_stamp: Option<Instant>,
    pub bomb_plant_timer: f32,
    pub bomb_being_defused: bool,
    pub bomb_defuse_stamp: Option<Instant>,
    pub bomb_defuse_length: f32,
    pub bomb_exploded: bool,
    pub bomb_defused: bool,
}


impl CsData {
    pub fn update_bomb(&mut self, ctx: &mut DmaCtx) {
        if self.bomb_dropped {
            // If the bomb is dropped, do a reverse entity list loop with early exit when we found the bomb.

            // We search in chunks of 64 indexes
            for chunk in &(0..=self.highest_index).rev().into_iter().chunks(64) {
                // data vec: (address, index, entity_identity_ptr, designer_name_ptr, designer_name_buff)
                let mut data_vec: Vec<(u64, i32, u64, u64, [u8; 2])> = chunk
                    .map(|idx| (0u64, idx, 0u64, 0u64, [0u8; 2]))
                    .collect();

                // Get the entity handle
                let mut batcher = ctx.process.batcher();
                data_vec.iter_mut().for_each(|(handle, idx, _, _, _)| {
                    let base: Address = (self.entity_list).into();
                    batcher.read_into(base + 8 * (*idx >> 9) + 16, handle);
                });
                drop(batcher);

                // Get the actual entity address
                let mut batcher = ctx.process.batcher();
                data_vec.iter_mut().for_each(|(ptr, index, _, _, _)| {
                    let base: Address = (*ptr).into();
                    batcher.read_into(base + 120 * (*index & 0x1FF), ptr);
                });
                drop(batcher);

                // Get the entity identity address
                let mut batcher = ctx.process.batcher();
                data_vec.iter_mut().for_each(|(ptr, _, ent_ident_ptr, _, _)| {
                    let base: Address = (*ptr).into();
                    batcher.read_into(base + cs2dumper::client::CEntityInstance::m_pEntity, ent_ident_ptr);
                });
                drop(batcher);

                // Get the designer name address
                let mut batcher = ctx.process.batcher();
                data_vec.iter_mut().for_each(|(_, _, ent_ident_ptr, designer_name_ptr, _)| {
                    let base: Address = (*ent_ident_ptr).into();
                    batcher.read_into(base + cs2dumper::client::CEntityIdentity::m_designerName, designer_name_ptr);
                });
                drop(batcher);

                // Read out 2 bytes of the designer name
                let mut batcher = ctx.process.batcher();
                data_vec.iter_mut().for_each(|(_, _, _, designer_name_ptr, designer_name_buff)| {
                    let base: Address = (*designer_name_ptr).into();
                    batcher.read_into(base + 7, designer_name_buff);
                });
                drop(batcher);

                // Actually check for the right designer name
                let bomb = data_vec.into_iter().find(|(_, _, _, _, designer_name_buff)| {
                    designer_name_buff == "c4".as_bytes()
                });

                if let Some(bomb) = bomb {
                    self.bomb = bomb.0.into();
                    break;
                }
            }
        } else if self.bomb_planted {
            let bomb = ctx.get_plantedc4()
                .expect("Failed to get planted bomb");

            self.bomb = bomb;
        }
    }

    pub fn update_players(&mut self, ctx: &mut DmaCtx) {
        let mut list_entries = [0u64; 64];
        {
            let mut batcher = ctx.process.batcher();
            let ent_list: Address = self.entity_list.into();

            list_entries.iter_mut().enumerate().for_each(|(idx, data)| {
                let index = idx as i32;
                batcher.read_into(ent_list + 8 * (index >> 9) + 16, data);
            });
        }

        let mut player_ptrs = [0u64; 64];
        {
            let mut batcher = ctx.process.batcher();

            player_ptrs.iter_mut().enumerate().for_each(|(idx, data)| {
                let list_entry: Address = list_entries[idx].into();
                batcher.read_into(list_entry + 120 * (idx & 0x1FF), data);
            });
        }

        let mut new_players: Vec<u64> = Vec::new();
        player_ptrs
            .into_iter()
            .for_each(|ptr| {
                if ctx.is_cs_player_controller(ptr.into()).unwrap_or(false) {
                    new_players.push(ptr)
                }
            });

        let new_players: Vec<(Address, Address)> = new_players
            .into_iter()
            .map(Address::from)
            .filter(|ptr| !ptr.is_null())
            .filter(|ptr| *ptr != self.local.into())
            .map(|ptr| {
                let pawn = ctx.pawn_from_controller(ptr, self.entity_list.into()).unwrap();
                (ptr, pawn)
            })
            .filter(|(_, pawn)| pawn.is_some())
            .map(|(controller, pawn)| (controller, pawn.unwrap()))
            .collect();

        self.players = new_players;
    }

    pub fn update_common(&mut self, ctx: &mut DmaCtx) {
        let mut bomb_dropped = 0u8;
        let mut bomb_planted = 0u8;
        let mut map_ptr = 0u64;
        let mut bomb_being_defused = 0u8;
        let mut bomb_exploded = 0u8;
        let mut bomb_defused = 0u8;
        let mut freeze_period = 0u8;
        {
            // Globals
            let tick_count_addr = (self.globals + 0x40).into();
            let map_addr = (self.globals + 384).into();

            // Gamerules
            let bomb_dropped_addr = (self.gamerules + cs2dumper::client::C_CSGameRules::m_bBombDropped as u64).into();
            let bomb_planted_addr = (self.gamerules + cs2dumper::client::C_CSGameRules::m_bBombPlanted as u64).into();
            let total_rounds_addr = (self.gamerules + cs2dumper::client::C_CSGameRules::m_bFreezePeriod as u64).into();
            let round_start_count_addr = (self.gamerules + cs2dumper::client::C_CSGameRules::m_nRoundStartCount as u64).into();

            // Game Entity System
            let highest_index_addr = (self.game_ent_sys + cs2dumper::offsets::client_dll::dwGameEntitySystem_highestEntityIndex as u64).into();

            let mut batcher = ctx.process.batcher();
            batcher.read_into(
                ctx.client_module.base + cs2dumper::offsets::client_dll::dwLocalPlayerController, 
                &mut self.local
            );
            batcher.read_into(
                ctx.client_module.base + cs2dumper::offsets::client_dll::dwLocalPlayerPawn, 
                &mut self.local_pawn
            );

            batcher.read_into(tick_count_addr, &mut self.tick_count);
            batcher.read_into(bomb_dropped_addr, &mut bomb_dropped);
            batcher.read_into(bomb_planted_addr, &mut bomb_planted);
            batcher.read_into(total_rounds_addr, &mut freeze_period);
            batcher.read_into(round_start_count_addr, &mut self.round_start_count);
            batcher.read_into(highest_index_addr, &mut self.highest_index);
            batcher.read_into(map_addr, &mut map_ptr);
        }

        {
            let mut batcher = ctx.process.batcher();
            if self.bomb_planted {

                batcher.read_into(self.bomb + cs2dumper::client::C_PlantedC4::m_flTimerLength , &mut self.bomb_plant_timer);
                batcher.read_into(self.bomb + cs2dumper::client::C_PlantedC4::m_bBombDefused, &mut bomb_defused);
                batcher.read_into(self.bomb + cs2dumper::client::C_PlantedC4::m_flDefuseLength, &mut self.bomb_defuse_length);
                batcher.read_into(self.bomb + cs2dumper::client::C_PlantedC4::m_bHasExploded, &mut bomb_exploded);
                batcher.read_into(self.bomb + cs2dumper::client::C_PlantedC4::m_bBeingDefused, &mut bomb_being_defused);

                drop(batcher);

                if self.bomb_planted_stamp.is_none() && !self.bomb.is_null() {
                    self.bomb_planted_stamp = Some(Instant::now());
                }

                if self.bomb_being_defused {
                    if self.bomb_defuse_stamp.is_none() {
                        self.bomb_defuse_stamp = Some(Instant::now())
                    }
                } else {
                    if self.bomb_defuse_stamp.is_some() {
                        self.bomb_defuse_stamp = None;
                    }
                }

            } else {
                if self.bomb_planted_stamp.is_some() {
                    self.bomb_planted_stamp = None;
                }

                if self.bomb_defuse_stamp.is_some() {
                    self.bomb_defuse_stamp = None;
                }
            }
        }


        let map_string = ctx.process.read_utf8_lossy(map_ptr.into(), 32).unwrap_or(String::from("<empty>"));

        self.map = map_string;
        self.bomb_dropped = bomb_dropped != 0;
        self.bomb_planted = bomb_planted != 0;
        self.bomb_exploded = bomb_exploded != 0;
        self.bomb_being_defused = bomb_being_defused != 0;
        self.bomb_defused = bomb_defused != 0;
        self.freeze_period = freeze_period != 0;
    }

    pub fn update_pointers(&mut self, ctx: &mut DmaCtx) {
        let mut batcher = ctx.process.batcher();
        batcher.read_into(ctx.client_module.base + cs2dumper::offsets::client_dll::dwGlobalVars, &mut self.globals);
        batcher.read_into(ctx.client_module.base + cs2dumper::offsets::client_dll::dwGameRules, &mut self.gamerules);
        batcher.read_into(ctx.client_module.base + cs2dumper::offsets::client_dll::dwEntityList, &mut self.entity_list);
        batcher.read_into(ctx.client_module.base + cs2dumper::offsets::client_dll::dwGameEntitySystem, &mut self.game_ent_sys);
    }
}