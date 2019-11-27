//! Behavior of the base enemy of the base. It will hover slowly
//! towards the player and shoot bullets at fixed interval.

use amethyst::{
    core::{math::Vector3, Transform},
    ecs::{Entities, LazyUpdate, Read, Write},
};

use crate::{
    config::SimpleEnemyConfig,
    systems::{animation::AnimationController, bullet::BulletSpawner, MyCollisionWorld},
};
use log::error;

#[derive(Debug, Clone, Copy)]
enum EnemyStatus {
    Walking,
    Shooting,
    PostShooting,
}

/// Data for the simple enemy.
#[derive(Debug, Clone, Copy)]
pub struct SimpleEnemy {
    /// How fast its bullet flies.
    bullet_speed: f32,

    /// how fast it walks.
    walk_speed: f32,

    /// what the enemy is currently doing
    state: EnemyStatus,

    /// State control
    /// How long have we been in the current state.
    current_state_duration: f32,

    /// How long does the enemy walk.
    walk_duration: f32,

    /// Shoot duration. How long does the enemy shoot? It is related to the shooting animation.
    shoot_duration: f32,
}

impl Default for SimpleEnemy {
    fn default() -> Self {
        Self {
            bullet_speed: 100.0,
            state: EnemyStatus::Walking,

            walk_speed: 0.2,
            current_state_duration: 0.0,
            walk_duration: 3.0,
            shoot_duration: 0.3,
        }
    }
}

impl SimpleEnemy {
    pub fn from_config(config: &SimpleEnemyConfig) -> Self {
        Self {
            bullet_speed: config.bullet_speed,
            walk_duration: config.walk_duration,
            shoot_duration: config.shoot_duration,
            ..SimpleEnemy::default()
        }
    }

    /// Simple enemy can be in three states. Either moving, shooting or dying.
    /// Each states has its own animations.
    /// Walking, it is just hovering towards the player.
    /// Shooting will take a few frames. First it opens the mouth and a bullet will come out of it.
    /// When the enemy has been killed, there is a death animation before the entity is removed.
    pub fn update(
        &mut self,
        delta_time: f32,
        t: &mut Transform,
        maybe_anim: &mut Option<&mut AnimationController>,
        player_vec: &Vector3<f32>,
        bullet_spawner: &Read<BulletSpawner>,
        entities: &Entities,
        updater: &Read<LazyUpdate>,
        collision: &mut Write<MyCollisionWorld>,
    ) {
        let enemy_vec = t.translation();

        let mut previous_animation = None;
        if let Some(ref mut anim) = maybe_anim {
            previous_animation = anim.current_animation.take();
        }
        let direction = player_vec - enemy_vec;

        //let dist = direction.norm();
        let d = direction.normalize();

        self.current_state_duration += delta_time;
        match self.state {
            EnemyStatus::Walking => {
                t.prepend_translation_x(self.walk_speed * d.x);
                t.prepend_translation_y(self.walk_speed * d.y);

                if let Some(ref mut anim) = maybe_anim {
                    if d.x < 0.0 {
                        anim.current_animation = Some("walk_left".to_string());
                    } else if d.x > 0.0 {
                        anim.current_animation = Some("walk_right".to_string());
                    } else if d.y > 0.0 {
                        anim.current_animation = Some("walk_up".to_string());
                    } else if d.y < 0.0 {
                        anim.current_animation = Some("walk_down".to_string());
                    }
                }

                // state transition if walked too long
                if self.current_state_duration >= self.walk_duration {
                    self.state = EnemyStatus::Shooting;
                    self.current_state_duration = 0.0;
                }
            }

            EnemyStatus::Shooting => {
                // shoot at the player :D
                if let Err(e) = bullet_spawner.spawn_enemy_bullet(
                    entities,
                    updater,
                    collision,
                    0,
                    *t.translation(),
                    direction.xy(),
                    self.bullet_speed,
                ) {
                    error!("Enemy cannot spawn bullet: {}", e);
                }

                // shoot animation is only visible when the enemy is facing down...
                if let Some(ref mut anim) = maybe_anim {
                    match previous_animation {
                        Some(ref x) if x.as_str() == "walk_up" => (),
                        _ => anim.current_animation = Some("shoot".to_string()),
                    }
                }

                self.current_state_duration = 0.0;
                self.state = EnemyStatus::PostShooting;
            }

            EnemyStatus::PostShooting => {
                if let Some(ref mut anim) = maybe_anim {
                    match previous_animation {
                        Some(ref x) if x.as_str() == "walk_up" => (),
                        _ => anim.current_animation = Some("shoot".to_string()),
                    }
                }

                if self.current_state_duration >= self.shoot_duration {
                    self.current_state_duration = 0.0;
                    self.state = EnemyStatus::Walking;
                }
            }
        }
    }
}
