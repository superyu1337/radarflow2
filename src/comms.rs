use serde::{Serialize, Deserialize};

use crate::{structs::Vec3, enums::PlayerType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub pos: Vec3,
    pub yaw: f32,
    #[serde(rename = "playerType")]
    pub player_type: PlayerType,
    #[serde(rename = "hasBomb")]
    pub has_bomb: bool,
    #[serde(rename = "hasAwp")]
    pub has_awp: bool,
    #[serde(rename = "isScoped")]
    pub is_scoped: bool
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BombData {
    /// If the bomb is planted
    pub planted: bool,

    /// If the bomb is dropped
    pub dropped: bool,

    /// Position of the bomb
    pub pos: Vec3,

    /// If the bomb was defused
    pub defused: bool,

    /// If the bomb exploded
    pub exploded: bool,

    /// If the bomb is being defused
    #[serde(rename = "beingDefused")]
    pub being_defused: bool,

    /// If the current defuse can be done before the bomb explodes
    #[serde(rename = "canDefuse")]
    pub can_defuse: bool,

    /// Current defuse length (either 10 or 5)
    #[serde(rename = "defuseLength")]
    pub defuse_length: f32,

    /// Time left until bomb explodes
    #[serde(rename = "timeleft")]
    pub timeleft: f32,

    /// Time till defuse ends
    #[serde(rename = "defuseEnd")]
    pub defuse_end: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarData {
    /// Frequency at which the DMA thread polls info
    freq: usize,
    ingame: bool,
    map_name: String,
    player_data: Vec<PlayerData>,
    bomb_data: BombData,
}

impl RadarData {
    pub fn new(freq: usize, ingame: bool, map_name: String, player_data: Vec<PlayerData>, bomb_data: BombData) -> Self {
        Self {
            freq,
            ingame,
            map_name,
            player_data,
            bomb_data,
        }
    }

    /// Returns empty RadarData, it's also the same data that gets sent to clients when not ingame
    pub fn empty(freq: usize) -> Self {
        Self {
            freq,
            ingame: false,
            map_name: String::new(),
            player_data: Vec::new(),
            bomb_data: BombData::default(),
        }
    }
}

pub type ArcRwlockRadarData = std::sync::Arc<tokio::sync::RwLock<RadarData>>;