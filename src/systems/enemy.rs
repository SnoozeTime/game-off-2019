//! System that control the enemies
//!
use amethyst::{
    core::{timing::Time, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::{
        Component, Entities, Join, LazyUpdate, Read, System, SystemData, VecStorage, World, Write,
        WriteStorage,
    },
};

use crate::systems::{AnimationController, BulletSpawner, MyCollisionWorld, PlayerResource};
#[allow(unused_imports)]
use log::{error, info};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum EnemyType {
    /// Most basic enemy. Follow and shoot.
    Simple,
}

/// Enemy component.
/// TODO enum with enemy types & behavior
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Enemy {
    /// Time before the enemy can shoot.
    time_before_shooting: f32,
    /// Number of sec the enemy will wait before shooting next.
    reload_time: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            time_before_shooting: 0.0,
            reload_time: 1.0,
        }
    }
}

#[derive(SystemDesc)]
pub struct EnemySystem;

impl<'s> System<'s> for EnemySystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Enemy>,
        Read<'s, PlayerResource>,
        Read<'s, Time>,
        Entities<'s>,
        Read<'s, LazyUpdate>,
        Read<'s, BulletSpawner>,
        Write<'s, MyCollisionWorld>,
        WriteStorage<'s, AnimationController>,
    );

    fn run(
        &mut self,
        (
            mut transforms,
            mut enemies,
            player,
            time,
            entities,
            updater,
            bullet_spawner,
            mut collision,
            mut animations,
        ): Self::SystemData,
    ) {
        if let Some(e) = player.player {
            if let Some(player_transform) = transforms.get(e).cloned() {
                let player_vec = player_transform.translation();

                for (t, enemy, e) in (&mut transforms, &mut enemies, &entities).join() {
                    let delta_time = time.delta_seconds();
                    let enemy_vec = t.translation();

                    let mut maybe_anim = animations.get_mut(e);
                    if let Some(ref mut anim) = maybe_anim {
                        anim.current_animation = None;
                    }
                    let direction = player_vec - enemy_vec;

                    let dist = direction.norm();
                    let d = direction.normalize();
                    if dist <= 150.0 {
                        if enemy.time_before_shooting <= 0.0 {
                            // shoot at the player :D
                            if let Err(e) = bullet_spawner.spawn_enemy_bullet(
                                &entities,
                                &updater,
                                &mut collision,
                                0,
                                *t.translation(),
                                direction.xy(),
                                100.0,
                            ) {
                                error!("Enemy cannot spawn bullet: {}", e);
                            }
                            enemy.time_before_shooting = enemy.reload_time;
                        } else {
                            enemy.time_before_shooting -= delta_time;
                        }
                    } else {
                        t.prepend_translation_x(1.2 * d.x);
                        t.prepend_translation_y(1.2 * d.y);

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
                    }
                }
            }
        }
        // info!("Processed data for {} enemies", i);
    }
}
