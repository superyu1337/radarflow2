use csflow::{enums::PlayerType, structs::Vec3};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pos: Vec3,
    yaw: f32,
    #[serde(rename = "playerType")]
    player_type: PlayerType,

    #[serde(rename = "hasBomb")]
    has_bomb: bool
}

impl PlayerData {
    pub fn new(pos: Vec3, yaw: f32, player_type: PlayerType, has_bomb: bool) -> PlayerData {
        PlayerData { pos, yaw, player_type, has_bomb }
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
    ingame: bool,

    #[serde(rename = "mapName")]
    map_name: String,

    #[serde(rename(serialize = "entityData"))]
    player_data: Vec<EntityData>,

    //#[serde(rename(serialize = "localYaw"))]
    //local_yaw: f32,
}

impl RadarData {
    pub fn new(ingame: bool, map_name: String, player_data: Vec<EntityData>) -> RadarData {
        RadarData { ingame, map_name, player_data }
    }

    /// Returns empty RadarData, it's also the same data that gets sent to clients when not ingame
    pub fn empty() -> RadarData {
        RadarData { 
            ingame: false,
            map_name: String::new(),
            player_data: Vec::new(),
        }
    }
}