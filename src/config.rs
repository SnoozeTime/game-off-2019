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
    pub creepy_boss: CreepyFirstBossConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct SimpleEnemyConfig {
    pub bullet_speed: f32,

    /// How long does the enemy walk.
    pub walk_duration: f32,

    /// Shoot duration. How long does the enemy shoot? It is related to the shooting animation.
    pub shoot_duration: f32,

    /// how fast the enemy walk.
    pub walk_speed: f32,

    pub health: i32,
}

impl Default for SimpleEnemyConfig {
    fn default() -> Self {
        Self {
            bullet_speed: 100.0,
            walk_duration: 2.0,
            shoot_duration: 1.0,
            walk_speed: 0.2,
            health: 2,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct CreepyFirstBossConfig {
    pub health: i32,
    pub collider_size: f32,
    /// nb of places where the boss will shoot the bullet.
    /// Distributed around him
    pub bullet_spawn: usize,
}

impl Default for CreepyFirstBossConfig {
    fn default() -> Self {
        Self {
            health: 10,
            collider_size: 48.0,
            bullet_spawn: 4,
        }
    }
}

// ---------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct BulletConfig {
    pub simple_enemy: SimpleBulletConfig,
    pub rotating_bullet: RotatingBulletConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default)]
pub struct SimpleBulletConfig;

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct RotatingBulletConfig {
    rot_per_sec: f32,
}

impl Default for RotatingBulletConfig {
    fn default() -> Self {
        Self { rot_per_sec: 1.0 }
    }
}
