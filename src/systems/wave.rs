//! This system will keep track of the current progression
//! of the player throw the waves.
//!
//! One wave contains a certain number of enemies. Once all waves are finished, it is time to move
//! to the next arena.
//!
//!
//! Wave manager will try to spawn enemies in a reasonable timing (not all enemies at the same time
//! for example).
//!
//!
//! The current arena config is in the config folder. It is loaded at
//! the state start and the file name is determined from the tiled map
//! properties.
//!

use crate::config::{ArenaConfig, WaveConfig};
use amethyst::ecs::{Component, VecStorage};
use log::error;

/// The component that will hold the state of the current arena waves.
/// Should have only one at a time.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Waves {
    waves: Vec<Wave>,
    current_wave: Option<usize>,
}

impl Waves {
    /// Will create the waves from a configuration file.
    pub fn from_config(config: ArenaConfig) -> Self {
        Self {
            waves: config
                .waves
                .iter()
                .cloned()
                .map(Wave::from_config)
                .collect(),
            current_wave: if config.waves.len() > 0 {
                Some(0)
            } else {
                error!("No waves in the configuration file");
                None
            },
        }
    }
}

/// not a component. Hold the current wave status.
#[derive(Debug, Clone, Copy)]
pub struct Wave {
    /// how many enemies we still have to spawn
    enemies_left: i32,

    /// How many enemies are currently in the arena
    /// When enemies are falling below a threshold, the wave system will
    /// spawn more.
    enemies_in_fly: i32,
}

impl Wave {
    fn from_config(config: WaveConfig) -> Self {
        Self {
            enemies_in_fly: config.enemies_in_fly,
            enemies_left: config.total_enemies,
        }
    }
}
