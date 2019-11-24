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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArenaConfig {
    pub waves: Vec<WaveConfig>,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self { waves: vec![] }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct WaveConfig {
    pub enemies_in_fly: i32,
    pub total_enemies: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct CameraConfig {
    pub ratio: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self { ratio: 1.0 }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct EnemyConfig {
    pub simple_enemy: SimpleEnemyConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct SimpleEnemyConfig {
    pub reload_time: f32,
    pub bullet_speed: f32,
}

impl Default for SimpleEnemyConfig {
    fn default() -> Self {
        Self {
            reload_time: 1.0,
            bullet_speed: 100.0,
        }
    }
}
