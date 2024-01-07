use itertools::Itertools;
use memflow::{mem::MemoryView, types::Address};

use super::{context::DmaCtx, cs2dumper};

#[derive(Clone, Debug, Default)]
pub struct CsData {
    // Entities
    pub players: Vec<(Address, Address)>,
    pub bomb: Address,
    pub bomb_holder: Address,
    pub recheck_bomb_holder: bool,

    // Pointers
    pub globals: u64,
    pub gamerules: u64,
    pub entity_list: u64,
    pub game_ent_sys: u64,

    // Common
    pub local: u64,
    pub local_pawn: u64,
    pub is_dead: bool,
    pub tick_count: i32,
    pub bomb_dropped: bool,
    pub bomb_planted: bool,
    pub highest_index: i32,
    pub map: String
}


impl CsData {
    pub fn update_bomb(&mut self, ctx: &mut DmaCtx) {
        // If the bomb is dropped, do a reverse entity list loop with early exit when we found the bomb. ( Now with BATCHING!!! :O )
        if self.bomb_dropped {

            // We search 16 entities at a time.
            for chunk in &(0..=self.highest_index).rev().chunks(16) {

                let indexes: Vec<i32> = chunk.collect();

                let mut data_array = [(0u64, 0i32); 16];
                {
                    let mut batcher = ctx.process.batcher();
                    let ent_list: Address = self.entity_list.into();

                    data_array.iter_mut().zip(indexes).for_each(|((data_ptr, data_idx), index)| {
                        batcher.read_into(ent_list + 8 * (index >> 9) + 16, data_ptr);
                        *data_idx = index;
                    });
                }

                {
                    let mut batcher = ctx.process.batcher();
    
                    data_array.iter_mut().for_each(|(ptr, index)| {
                        let handle: Address = (*ptr).into();
                        batcher.read_into(handle + 120 * (*index & 0x1FF), ptr);
                    });
                }

                // You can actually optimize this EVEN more
                let bomb = data_array.into_iter().find(|(ptr, _)| {
                    // By doing this with a batcher too...
                    ctx.is_dropped_c4((*ptr).into()).unwrap_or(false)
                });

                if let Some(bomb) = bomb {
                    self.bomb = bomb.0.into();
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

        {
            // Globals
            let tick_count_addr = (self.globals + 0x40).into();
            let map_addr = (self.globals + 0x188).into();

            // Gamerules
            let bomb_dropped_addr = (self.gamerules + cs2dumper::client::C_CSGameRules::m_bBombDropped as u64).into();
            let bomb_planted_addr = (self.gamerules + cs2dumper::client::C_CSGameRules::m_bBombPlanted as u64).into();

            // Game Entity System
            let highest_index_addr = (self.game_ent_sys + cs2dumper::offsets::client_dll::dwGameEntitySystem_getHighestEntityIndex as u64).into();

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
            batcher.read_into(highest_index_addr, &mut self.highest_index);
            batcher.read_into(map_addr, &mut map_ptr);
        }

        let map_string = ctx.process.read_char_string_n(map_ptr.into(), 32).unwrap_or(String::from("<empty>"));

        self.map = map_string;
        self.bomb_dropped = bomb_dropped != 0;
        self.bomb_planted = bomb_planted != 0;
    }

    pub fn update_pointers(&mut self, ctx: &mut DmaCtx) {
        let mut batcher = ctx.process.batcher();
        batcher.read_into(ctx.client_module.base + cs2dumper::offsets::client_dll::dwGlobalVars, &mut self.globals);
        batcher.read_into(ctx.client_module.base + cs2dumper::offsets::client_dll::dwGameRules, &mut self.gamerules);
        batcher.read_into(ctx.client_module.base + cs2dumper::offsets::client_dll::dwEntityList, &mut self.entity_list);
        batcher.read_into(ctx.client_module.base + cs2dumper::offsets::client_dll::dwGameEntitySystem, &mut self.game_ent_sys);
    }
}