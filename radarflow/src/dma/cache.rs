use csflow::{memflow::Address, enums::PlayerType, structs::{GlobalVars, GameRules}, traits::MemoryClass};

#[derive(Clone, Copy)]
pub enum CachedEntity {
    Bomb {ptr: Address},
    Player {ptr: Address, player_type: PlayerType},
}

pub struct Cache {
    timestamp: std::time::Instant,
    valid: bool,
    entity_cache: Vec<CachedEntity>,
    entity_list: Address,
    globals: GlobalVars,
    gamerules: GameRules,
}

impl Cache {
    pub fn is_valid(&self) -> bool {
        if self.valid {
            if self.timestamp.elapsed() > std::time::Duration::from_secs(60 * 3) {
                log::info!("Invalidated cache! Reason: time");
                return false
            }

            true
        } else { false }
    }

    pub fn new_invalid() -> Cache {
        Cache {
            timestamp: std::time::Instant::now().checked_sub(std::time::Duration::from_millis(500)).unwrap(),
            valid: false,
            entity_cache: Vec::new(),
            entity_list: Address::null(),
            globals: GlobalVars::new(Address::null()),
            gamerules: GameRules::new(Address::null()),
        }
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    pub fn entity_cache(&mut self) -> Vec<CachedEntity> {
        self.entity_cache.clone()
    }

    pub fn entity_list(&self) -> Address {
        self.entity_list
    }

    pub fn globals(&self) -> GlobalVars {
        self.globals
    }

    pub fn gamerules(&self) -> GameRules {
        self.gamerules
    }
}

pub struct CacheBuilder {
    entity_cache: Option<Vec<CachedEntity>>,
    entity_list: Option<Address>,
    globals: Option<GlobalVars>,
    gamerules: Option<GameRules>
}

impl CacheBuilder {
    pub fn new() -> CacheBuilder {
        CacheBuilder {
            entity_cache: None,
            entity_list: None,
            globals: None,
            gamerules: None,
        }
    }

    pub fn entity_cache(mut self, entity_cache: Vec<CachedEntity>) -> CacheBuilder {
        self.entity_cache = Some(entity_cache);
        self
    }

    pub fn entity_list(mut self, entity_list: Address) -> CacheBuilder {
        self.entity_list = Some(entity_list);
        self
    }

    pub fn globals(mut self, globals: GlobalVars) -> CacheBuilder {
        self.globals = Some(globals);
        self
    }

    pub fn gamerules(mut self, gamerules: GameRules) -> CacheBuilder {
        self.gamerules = Some(gamerules);
        self
    }

    pub fn build(self) -> anyhow::Result<Cache> {
        Ok(Cache {
            timestamp: std::time::Instant::now(),
            valid: true,
            entity_cache: self.entity_cache.ok_or(anyhow::anyhow!("entity_cache not set on builder"))?,
            entity_list: self.entity_list.ok_or(anyhow::anyhow!("entity_list not set on builder"))?,
            globals: self.globals.ok_or(anyhow::anyhow!("globals not set on builder"))?,
            gamerules: self.gamerules.ok_or(anyhow::anyhow!("gamerules not set on builder"))?,
        })
    }
}