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

use crate::systems::{BulletSpawner, MyCollisionWorld, PlayerResource};
use log::error;

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
        ): Self::SystemData,
    ) {
        if let Some(e) = player.player {
            let player_transform = transforms
                .get(e)
                .expect("player should have a transform")
                .clone();
            let player_vec = player_transform.translation();

            for (t, enemy) in (&mut transforms, &mut enemies).join() {
                let delta_time = time.delta_seconds();
                let enemy_vec = t.translation();

                let direction = player_vec - enemy_vec;

                let dist = direction.norm();
                let d = direction.normalize();
                if dist <= 150.0 {
                    if enemy.time_before_shooting <= 0.0 {
                        // shoot at the player :D
                        if let Err(e) = bullet_spawner.spawn_bullet(
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
                }
            }
        }
    }
}
