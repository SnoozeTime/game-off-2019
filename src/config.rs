use serde::{Deserialize, Serialize};

/// Load configuration from config file (ronronron)
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct PlayerConfig {
    pub fall_rot_speed: f32,
    pub player_speed: f32,
    pub health: i32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            player_speed: 50.0,
            health: 5,
            fall_rot_speed: 25.0,
        }
    }
}
