use memflow::{types::Address, mem::MemoryView};
use crate::{traits::MemoryClass, CheatCtx, Error, cs2dumper};

#[derive(Debug, Clone, Copy)]
pub struct GameRules(Address);

impl MemoryClass for GameRules {
    fn ptr(&self) -> memflow::types::Address {
        self.0
    }

    fn new(ptr: memflow::types::Address) -> Self {
        Self(ptr)
    }
}

impl GameRules {
    pub fn bomb_dropped(&self, ctx: &mut CheatCtx) -> Result<bool, Error> {
        let data: u8 = ctx.memory.read(self.0 + cs2dumper::client::C_CSGameRules::m_bBombDropped)?;
        Ok(data != 0)
    }

    pub fn total_rounds_played(&self, ctx: &mut CheatCtx) -> Result<i32, Error> {
        Ok(ctx.memory.read(self.0 + cs2dumper::client::C_CSGameRules::m_totalRoundsPlayed)?)
    }

    pub fn game_phase(&self, ctx: &mut CheatCtx) -> Result<i32, Error> {
        Ok(ctx.memory.read(self.0 + cs2dumper::client::C_CSGameRules::m_gamePhase)?)
    }

    pub fn bomb_planted(&self, ctx: &mut CheatCtx) -> Result<bool, Error> {
        let data: u8 = ctx.memory.read(self.0 + cs2dumper::client::C_CSGameRules::m_bBombPlanted)?;
        Ok(data != 0)
    }
}