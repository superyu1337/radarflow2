use memflow::types::Address;

use crate::structs::communication::PlayerType;

#[derive(Clone, Copy)]
pub enum CachedEntity {
    Bomb {ptr: Address},
    Player {ptr: Address, player_type: PlayerType},
}

pub struct Cache {
    timestamp: std::time::Instant,
    entity_cache: Vec<CachedEntity>,
    map_name: String,
    entity_list: Address,
}

impl Cache {
    pub fn is_valid(&self) -> bool {
        if self.timestamp.elapsed() > std::time::Duration::from_millis(250) {
            return false;
        }

        true
    }

    pub fn new_invalid() -> Cache {
        Cache {
            timestamp: std::time::Instant::now().checked_sub(std::time::Duration::from_millis(500)).unwrap(),
            entity_cache: Vec::new(),
            map_name: String::new(),
            entity_list: Address::null(),
        }
    }

    pub fn entity_cache(&mut self) -> Vec<CachedEntity> {
        self.entity_cache.clone()
    }

    pub fn map_name(&self) -> String {
        self.map_name.clone()
    }

    pub fn entity_list(&self) -> Address {
        self.entity_list
    }
}

pub struct CacheBuilder {
    entity_cache: Option<Vec<CachedEntity>>,
    map_name: Option<String>,
    entity_list: Option<Address>
}

impl CacheBuilder {
    pub fn new() -> CacheBuilder {
        CacheBuilder {
            entity_cache: None,
            map_name: None,
            entity_list: None,
        }
    }

    pub fn entity_cache(mut self, entity_cache: Vec<CachedEntity>) -> CacheBuilder {
        self.entity_cache = Some(entity_cache);
        self
    }

    pub fn map_name(mut self, map_name: String) -> CacheBuilder {
        self.map_name = Some(map_name);
        self
    }

    pub fn entity_list(mut self, entity_list: Address) -> CacheBuilder {
        self.entity_list = Some(entity_list);
        self
    }

    pub fn build(self) -> anyhow::Result<Cache> {
        Ok(Cache {
            timestamp: std::time::Instant::now(),
            entity_cache: self.entity_cache.ok_or(anyhow::anyhow!("entity_cache not set on builder"))?,
            map_name: self.map_name.ok_or(anyhow::anyhow!("map_name not set on builder"))?,
            entity_list: self.entity_list.ok_or(anyhow::anyhow!("entity_list not set on builder"))?,
        })
    }
}