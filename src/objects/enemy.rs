//! Helpers to create the enemy entities...
//!
use crate::{
    systems::{enemy::EnemyType, Collider, ColliderObjectType, Enemy, MyCollisionWorld},
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

/// Enemy spawner will help creating new enemies. It hold the necessary asset handles (e.g.
/// textures, animation,...) necessary to create new enemies.
#[derive(Debug, Default)]
pub struct EnemySpawner {
    textures: HashMap<EnemyType, Handle<SpriteSheet>>,
}

impl EnemySpawner {
    ///
    pub fn init(world: &mut World) -> Self {
        // default implementation assumes all assets are present. Doesn't make sense otherwise...
        let mut textures = HashMap::new();
        let sprite_sheet = load_spritesheet("rectangle", world);
        textures.insert(EnemyType::Simple, sprite_sheet);
        Self { textures }
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

            let collider = {
                let collider = Collider::new_rect(
                    Vector2::new(0.0, 0.0),
                    16.0,
                    16.0,
                    &mut collision.world,
                    ColliderObjectType::Enemy,
                    Some(entity),
                );
                //collider.set_entity(&mut collision.world, entity);
                collider
            };

            updater.insert(entity, position);
            updater.insert(entity, sprite);
            updater.insert(entity, Enemy::default());
            updater.insert(entity, collider);
            Some(entity)
        } else {
            None
        }
    }
}