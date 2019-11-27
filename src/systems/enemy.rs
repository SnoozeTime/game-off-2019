//! System that control the enemies
//!
use amethyst::{
    core::{math::Vector3, timing::Time, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::{
        Component, Entities, Join, LazyUpdate, Read, System, SystemData, VecStorage, World, Write,
        WriteStorage,
    },
};

use crate::objects::enemy::{CreepyFirstBoss, SimpleEnemy};
use crate::{
    config::EnemyConfig,
    systems::{AnimationController, BulletSpawner, MyCollisionWorld, PlayerResource},
};

#[allow(unused_imports)]
use log::{error, info};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum EnemyType {
    /// Most basic enemy. Follow and shoot.
    Simple,
    CreepyFirstBoss,
}

#[derive(Debug, Clone, Copy, Component)]
#[storage(VecStorage)]
pub enum Enemy {
    Simple(SimpleEnemy),
    CreepyFirstBoss(CreepyFirstBoss),
}

impl Enemy {
    pub fn from_config(enemy_type: EnemyType, config: &EnemyConfig) -> Self {
        match enemy_type {
            EnemyType::Simple => Enemy::Simple(SimpleEnemy::from_config(&config.simple_enemy)),
            EnemyType::CreepyFirstBoss => Enemy::CreepyFirstBoss(CreepyFirstBoss::default()),
        }
    }

    /// What to do every frame
    pub fn update(
        &mut self,
        delta_time: f32,
        t: &mut Transform,
        animations: &mut Option<&mut AnimationController>,

        player_vec: &Vector3<f32>,
        bullet_spawner: &Read<BulletSpawner>,
        entities: &Entities,
        updater: &Read<LazyUpdate>,
        collision: &mut Write<MyCollisionWorld>,
    ) {
        match *self {
            Enemy::Simple(ref mut simple_enemy) => simple_enemy.update(
                delta_time,
                t,
                animations,
                player_vec,
                bullet_spawner,
                entities,
                updater,
                collision,
            ),
            Enemy::CreepyFirstBoss(ref mut creepy_boss) => {
                creepy_boss.update(delta_time, t, bullet_spawner, entities, updater, collision)
            }
        }
    }
}

impl Default for Enemy {
    fn default() -> Self {
        Enemy::Simple(SimpleEnemy::default())
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
                    enemy.update(
                        time.delta_seconds(),
                        t,
                        &mut animations.get_mut(e),
                        &player_vec,
                        &bullet_spawner,
                        &entities,
                        &updater,
                        &mut collision,
                    );
                }
            }
        }
        // info!("Processed data for {} enemies", i);
    }
}
