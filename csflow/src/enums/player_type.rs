#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Default, PartialEq)]
pub enum PlayerType {
    #[default]
    Unknown,
    Spectator,
    Local,
    Enemy,
    Team
}