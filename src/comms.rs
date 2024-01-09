use serde::{Serialize, Deserialize};

use crate::{structs::Vec3, enums::PlayerType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pos: Vec3,
    yaw: f32,
    #[serde(rename = "playerType")]
    player_type: PlayerType,

    #[serde(rename = "hasBomb")]
    has_bomb: bool,

    #[serde(rename = "hasAwp")]
    has_awp: bool,

    #[serde(rename = "isScoped")]
    is_scoped: bool
}

impl PlayerData {
    pub fn new(pos: Vec3, yaw: f32, player_type: PlayerType, has_bomb: bool, has_awp: bool, is_scoped: bool) -> PlayerData {
        PlayerData { pos, yaw, player_type, has_bomb, has_awp, is_scoped }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BombData {
    pos: Vec3,
    #[serde(rename = "isPlanted")]
    is_planted: bool
}

#[allow(dead_code)]
impl BombData {
    pub fn new(pos: Vec3, is_planted: bool) -> BombData {
        BombData { pos, is_planted }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityData {
    Player(PlayerData),
    Bomb(BombData)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarData {
    freq: usize,
    ingame: bool,

    #[serde(rename = "bombPlanted")]
    bomb_planted: bool,

    #[serde(rename = "bombExploded")]
    bomb_exploded: bool,

    #[serde(rename = "bombBeingDefused")]
    bomb_being_defused: bool,

    #[serde(rename = "bombCanDefuse")]
    bomb_can_defuse: bool,

    #[serde(rename = "bombDefuseLength")]
    bomb_defuse_length: f32,

    #[serde(rename = "bombDefuseTimeleft")]
    bomb_defuse_timeleft: f32,

    #[serde(rename = "mapName")]
    map_name: String,

    #[serde(rename(serialize = "entityData"))]
    player_data: Vec<EntityData>,

    //#[serde(rename(serialize = "localYaw"))]
    //local_yaw: f32,
}

impl RadarData {
    pub fn new(ingame: bool, map_name: String, player_data: Vec<EntityData>, freq: usize, bomb_planted: bool, bomb_cannot_defuse: bool, bomb_defuse_timeleft: f32, bomb_exploded: bool, bomb_being_defused: bool, bomb_defuse_length: f32) -> RadarData {
        RadarData { ingame, map_name, player_data, freq, bomb_planted, bomb_can_defuse: bomb_cannot_defuse, bomb_defuse_timeleft, bomb_exploded, bomb_being_defused, bomb_defuse_length }
    }

    /// Returns empty RadarData, it's also the same data that gets sent to clients when not ingame
    pub fn empty(freq: usize) -> RadarData {
        RadarData { 
            ingame: false,
            map_name: String::new(),
            player_data: Vec::new(),
            freq,
            bomb_planted: false,
            bomb_can_defuse: false,
            bomb_defuse_timeleft: 0.0,
            bomb_exploded: false,
            bomb_being_defused: false,
            bomb_defuse_length: 0.0
        }
    }
}

unsafe impl Send for RadarData {}

pub type ArcRwlockRadarData = std::sync::Arc<tokio::sync::RwLock<RadarData>>;