//! CreepyFirstBoss is the creepy stationary big blob.
//! it is throwing bunch of bullets at the player since it cannot move.
//!
//! I has two status. Shooting a bunch of bullets and waiting.
//!

use amethyst::{
    core::{
        math::{Vector2, Vector3},
        Transform,
    },
    ecs::{Entities, LazyUpdate, Read, Write},
};

use crate::{
    config::SimpleEnemyConfig,
    systems::{animation::AnimationController, bullet::BulletSpawner, MyCollisionWorld},
};
use log::error;

#[derive(Debug, Clone, Copy)]
enum EnemyStatus {
    Shooting,
    Waiting,
}

#[derive(Debug, Clone, Copy)]
pub struct CreepyFirstBoss {
    state: EnemyStatus,
    current_state_duration: f32,
    shoot_duration: f32,
    wait_duration: f32,

    frames_between_shot: i32,
    nb_frame: i32,
}

impl Default for CreepyFirstBoss {
    fn default() -> Self {
        Self {
            state: EnemyStatus::Waiting,
            current_state_duration: 0.0,
            shoot_duration: 3.0,
            wait_duration: 2.0,
            frames_between_shot: 10,
            nb_frame: 0,
        }
    }
}

impl CreepyFirstBoss {
    pub fn update(
        &mut self,
        delta_time: f32,
        t: &mut Transform,
        bullet_spawner: &Read<BulletSpawner>,
        entities: &Entities,
        updater: &Read<LazyUpdate>,
        collision: &mut Write<MyCollisionWorld>,
    ) {
        self.current_state_duration += delta_time;
        match self.state {
            EnemyStatus::Waiting => {
                if self.current_state_duration >= self.wait_duration {
                    self.state = EnemyStatus::Shooting;
                    self.current_state_duration = 0.0;
                }
            }

            EnemyStatus::Shooting => {
                let shoot_loc = t.translation() + Vector3::new(0.0, -25.0, 0.0);
                if self.frames_between_shot == self.nb_frame {
                    // shoot at the player :D
                    if let Err(e) = bullet_spawner.spawn_enemy_bullet(
                        entities,
                        updater,
                        collision,
                        0,
                        shoot_loc,
                        Vector2::new(0.0, -1.0),
                        50.0,
                    ) {
                        error!("Enemy cannot spawn bullet: {}", e);
                    }

                    self.nb_frame = 0;
                } else {
                    self.nb_frame += 1;
                }

                if self.current_state_duration >= self.shoot_duration {
                    self.current_state_duration = 0.0;
                    self.state = EnemyStatus::Waiting;
                }
            }
        }
    }
}
