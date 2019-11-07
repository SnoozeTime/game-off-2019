use serde::{Deserialize, Serialize};

/// Load configuration from config file (ronronron)
#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerConfig {
    pub fall_rot_speed: f32,
    pub player_speed: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            player_speed: 50.0,
            fall_rot_speed: 25.0,
        }
    }
}
