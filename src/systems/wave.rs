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

use crate::{
    config::{ArenaConfig, WaveConfig},
    event::AppEvent,
};
use amethyst::{
    core::{
        shrev::{EventChannel, ReaderId},
        SystemDesc,
    },
    derive::SystemDesc,
    ecs::{Component, Join, System, SystemData, VecStorage, World, Write, WriteStorage},
};
#[allow(unused_imports)]
use log::{debug, error, info};

/// The component that will hold the state of the current arena waves.
/// Should have only one at a time.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Waves {
    waves: Vec<Wave>,
    current_wave: usize,
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
                0
            } else {
                panic!("No waves in the configuration file");
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

    /// If wave in idle status, the wave system will need to spwan all the enemies.
    status: WaveStatus,
}

#[derive(Debug, Clone, Copy)]
pub enum WaveStatus {
    Idle,
    Running,
    Over,
}

impl Wave {
    fn from_config(config: WaveConfig) -> Self {
        Self {
            enemies_in_fly: config.enemies_in_fly,
            enemies_left: config.total_enemies,
            status: WaveStatus::Idle,
        }
    }
}

/// Will keep track of how many enemies are in the arena and when to spawn
/// new enemies.
#[derive(SystemDesc)]
#[system_desc(name(WaveSystemDesc))]
pub struct WaveSystem {
    #[system_desc(event_channel_reader)]
    reader_id: ReaderId<AppEvent>,
}

impl WaveSystem {
    pub fn new(reader_id: ReaderId<AppEvent>) -> Self {
        Self { reader_id }
    }
}

impl<'s> System<'s> for WaveSystem {
    type SystemData = (WriteStorage<'s, Waves>, Write<'s, EventChannel<AppEvent>>);

    fn run(&mut self, (mut waves, mut events): Self::SystemData) {
        // poll the events even if there is no wave configured.. can we lose some
        // events? dunno...
        let mut to_spawn = 0;
        for ev in events.read(&mut self.reader_id) {
            if let AppEvent::EnemyDied(_) = ev {
                debug!("Enemy died :D");
                to_spawn += 1;
            }
        }

        // only one waves component.
        if let Some(ref mut waves) = (&mut waves).join().next() {
            if let Some(ref mut wave) = waves.waves.get_mut(waves.current_wave) {
                // Process the current wave
                match wave.status {
                    WaveStatus::Idle => {
                        debug!("Will initialize wave");
                        let enemy_to_spawn = wave.enemies_left.min(wave.enemies_in_fly);
                        events.single_write(AppEvent::SpawnEnemy(enemy_to_spawn));
                        wave.status = WaveStatus::Running;
                    }
                    WaveStatus::Running => {
                        //
                        if to_spawn > 0 {
                            wave.enemies_left = (wave.enemies_left - to_spawn).max(0);

                            if wave.enemies_left == 0 {
                                info!("WAVE FINISHED!!!");
                            } else {
                                let enemy_to_spawn = to_spawn.min(wave.enemies_left);
                                events.single_write(AppEvent::SpawnEnemy(enemy_to_spawn));
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}
