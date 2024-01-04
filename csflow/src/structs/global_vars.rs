use memflow::{types::Address, mem::MemoryView};
use crate::{traits::MemoryClass, CheatCtx, Error, cached_view::INVALIDATE_ALWAYS};

#[derive(Debug, Clone, Copy)]
pub struct GlobalVars(Address);

impl MemoryClass for GlobalVars {
    fn ptr(&self) -> memflow::types::Address {
        self.0
    }

    fn new(ptr: memflow::types::Address) -> Self {
        Self(ptr)
    }
}

impl GlobalVars {
    pub fn real_time(&self, ctx: &mut CheatCtx) -> Result<f32, Error> {
        ctx.cache_controller.set_next_flags(INVALIDATE_ALWAYS);
        Ok(ctx.process.read(self.0)?)
    }

    pub fn frame_count(&self, ctx: &mut CheatCtx) -> Result<i32, Error> {
        Ok(ctx.process.read(self.0 + 0x4)?)
    }

    pub fn frame_time(&self, ctx: &mut CheatCtx) -> Result<f32, Error> {
        Ok(ctx.process.read(self.0 + 0x8)?)
    }

    pub fn absolute_frame_time(&self, ctx: &mut CheatCtx) -> Result<f32, Error> {
        Ok(ctx.process.read(self.0 + 0xC)?)
    }

    pub fn max_clients(&self, ctx: &mut CheatCtx) -> Result<i32, Error> {
        Ok(ctx.process.read(self.0 + 0x10)?)
    }

    pub fn tick_count(&self, ctx: &mut CheatCtx) -> Result<i32, Error> {
        Ok(ctx.process.read(self.0 + 0x40)?)
    }

    pub fn map_name(&self, ctx: &mut CheatCtx) -> Result<String, Error> {
        let ptr = ctx.memory.read_addr64(self.0 + 0x188)?;
        Ok(ctx.process.read_char_string_n(ptr, 32)?)
    }
}

/* 
struct GlobalVarsBase {
    real_time: f32,                  // 0x0000
    frame_count: i32,                // 0x0004
    frame_time: f32,                 // 0x0008
    absolute_frame_time: f32,        // 0x000C
    max_clients: i32,                // 0x0010
    pad_0: [u8; 0x14],               // 0x0014
    frame_time_2: f32,               // 0x0028
    current_time: f32,               // 0x002C
    current_time_2: f32,             // 0x0030
    pad_1: [u8; 0xC],                // 0x0034
    tick_count: f32,                 // 0x0040 // NO fucking idea why the fuck this "should" be an f32????
    pad_2: [u8; 0x4],                // 0x0044
    network_channel: *const c_void,  // 0x0048
    pad_3: [u8; 0x130],              // 0x0050
    current_map: *const c_char,      // 0x0180
    current_map_name: *const c_char, // 0x0188
}
*/