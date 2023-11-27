mod entity;
pub use entity::*;
use memflow::types::Address;

use crate::structs::communication::PlayerType;

#[derive(Clone, Copy)]
pub enum CachedEntityData {
    Bomb {ptr: Address},
    Player {ptr: Address, player_type: PlayerType},
}

pub struct CommonCache {
    map_name: String,
    entity_list: Address,
}

impl CommonCache {
    pub fn new() -> CommonCache {
        CommonCache {
            map_name: String::from("unknown"),
            entity_list:  Address::null(),
        }
    }

    pub fn update(&mut self, map_name: String, entity_list: Address) {
        self.map_name = map_name;
        self.entity_list = entity_list;
    }

    pub fn map_name(&self) -> String {
        self.map_name.clone()
    }

    pub fn entity_list(&self) -> Address {
        self.entity_list
    }
}

pub struct Cache {
    last_cached: std::time::Instant,
    data: Vec<CachedEntityData>,
    common: CommonCache
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            last_cached: std::time::Instant::now().checked_sub(std::time::Duration::from_millis(500)).unwrap(),
            data: Vec::new(),
            common: CommonCache::new(),
        }
    }

    pub fn is_outdated(&self) -> bool {
        if self.last_cached.elapsed() > std::time::Duration::from_millis(250) {
            return true;
        }

        false
    }

    pub fn new_time(&mut self) {
        self.last_cached = std::time::Instant::now();
    }

    pub fn clean(&mut self) {
        self.data.clear();
    }

    pub fn data(&self) -> Vec<CachedEntityData> {
        self.data.clone()
    }

    pub fn push_data(&mut self, data: CachedEntityData) {
        self.data.push(data);
    }

    pub fn common(&mut self) -> &mut CommonCache {
        &mut self.common
    }
}