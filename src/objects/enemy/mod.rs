//! Helpers to create the enemy entities...
//!
use crate::{
    config::EnemyConfig,
    objects::animations,
    systems::{
        enemy::EnemyType, health::Health, AnimationController, Collider, ColliderObjectType, Enemy,
        MyCollisionWorld,
    },
    util::load_spritesheet,
};
use amethyst::{
    assets::Handle,
    core::{math::Vector2, Transform},
    ecs::{Entities, Entity, LazyUpdate, Read, Write},
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};
use std::collections::HashMap;

mod simple_enemy;
pub use simple_enemy::SimpleEnemy;
mod creepy_boss;
pub use creepy_boss::CreepyFirstBoss;

/// Enemy spawner will help creating new enemies. It hold the necessary asset handles (e.g.
/// textures, animation,...) necessary to create new enemies.
#[derive(Debug, Default)]
pub struct EnemySpawner {
    textures: HashMap<EnemyType, Handle<SpriteSheet>>,
    enemy_config: EnemyConfig,
}

impl EnemySpawner {
    ///
    pub fn init(world: &mut World) -> Self {
        // default implementation assumes all assets are present. Doesn't make sense otherwise...
        let mut textures = HashMap::new();
        let sprite_sheet = load_spritesheet("enemy_simple", world);
        let boss = load_spritesheet("boss_1", world);
        textures.insert(EnemyType::Simple, sprite_sheet);
        textures.insert(EnemyType::CreepyFirstBoss, boss);
        Self {
            textures,
            enemy_config: *world.read_resource::<EnemyConfig>(),
        }
    }

    /// Will spawn an enemy at the given position. This is when the user
    /// has access to the World object.
    pub fn create_enemy(
        &self,
        world: &mut World,
        enemy_type: EnemyType,
        position: Transform,
    ) -> Option<Entity> {
        let mut maybe_entity = None;
        world.exec(
            |(entities, updater, mut collision): (
                Entities,
                Read<LazyUpdate>,
                Write<MyCollisionWorld>,
            )| {
                maybe_entity =
                    self.spawn_enemy(&entities, &updater, &mut collision, enemy_type, position);
            },
        );
        maybe_entity
    }

    /// Will spawn an enemy. This is to be called from a system.
    pub fn spawn_enemy(
        &self,
        entities: &Entities,
        updater: &LazyUpdate,
        collision: &mut MyCollisionWorld,
        enemy_type: EnemyType,
        position: Transform,
    ) -> Option<Entity> {
        if let Some(handle) = self.textures.get(&enemy_type) {
            let sprite = SpriteRender {
                sprite_sheet: handle.clone(),
                sprite_number: 0,
            };

            let entity = entities.create();
            let walking_animations = animations::get_enemy_anim(enemy_type);
            if let Some(walking_animations) = walking_animations {
                let mut animation_controller = AnimationController {
                    animations: HashMap::new(),
                    current_animation: None,
                };
                animation_controller.animations.extend(walking_animations);
                updater.insert(entity, animation_controller);
            }
            updater.insert(entity, position);
            updater.insert(entity, sprite);
            updater.insert(entity, Enemy::from_config(enemy_type, &self.enemy_config));
            //updater.insert(entity, collider);
            self.add_collider(updater, entity, collision, enemy_type);
            self.add_health(updater, entity, enemy_type);
            Some(entity)
        } else {
            None
        }
    }

    fn add_health(&self, updater: &LazyUpdate, entity: Entity, enemy_type: EnemyType) {
        match enemy_type {
            EnemyType::CreepyFirstBoss => {
                updater.insert(entity, Health::new(self.enemy_config.creepy_boss.health))
            }

            EnemyType::Simple => {
                updater.insert(entity, Health::new(self.enemy_config.simple_enemy.health))
            }
        }
    }

    fn add_collider(
        &self,
        updater: &LazyUpdate,
        entity: Entity,
        collision: &mut MyCollisionWorld,
        enemy_type: EnemyType,
    ) {
        let w = if let EnemyType::CreepyFirstBoss = enemy_type {
            self.enemy_config.creepy_boss.collider_size
        } else {
            16.0
        };
        let collider = {
            let collider = Collider::new_rect(
                Vector2::new(0.0, 0.0),
                w,
                w,
                &mut collision.world,
                ColliderObjectType::Enemy,
                None,
                Some(entity),
            );
            //collider.set_entity(&mut collision.world, entity);
            collider
        };
        updater.insert(entity, collider);
    }
}
