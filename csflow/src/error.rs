use memflow::types::Address;
use thiserror::Error;

use crate::structs::Vec3;

#[derive(Error, Debug)]
pub enum Error {
    /// Game version mismatch.
    /// First arg is the game version, second is the offset version.
    #[error("version mismatch, game has version {0}, but offsets have version {1}")]
    GameVersionMismatch(usize, usize),

    #[error("memflow error: {0}")]
    Memflow(#[from] memflow::error::Error),

    #[error("memflow partial error when reading address: {0}")]
    MemflowPartialAddress(#[from] memflow::error::PartialError<Address>),

    #[error("memflow partial error when reading Vec3: {0}")]
    MemflowPartialVec3(#[from] memflow::error::PartialError<Vec3>),

    #[error("memflow partial error when reading String: {0}")]
    MemflowPartialString(#[from] memflow::error::PartialError<String>),

    #[error("memflow partial error when reading i32: {0}")]
    MemflowPartiali32(#[from] memflow::error::PartialError<i32>),

    #[error("memflow partial error when reading u32: {0}")]
    MemflowPartialu32(#[from] memflow::error::PartialError<u32>),

    #[error("memflow partial error when reading f32: {0}")]
    MemflowPartialf32(#[from] memflow::error::PartialError<f32>),

    #[error("memflow partial error when reading u8: {0}")]
    MemflowPartialu8(#[from] memflow::error::PartialError<u8>),

    #[error("memflow partial error when reading Vec<u8>: {0}")]
    MemflowPartialVecu8(#[from] memflow::error::PartialError<Vec<u8>>)
}