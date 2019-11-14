//! Helpers to create the player entity...
//!
use crate::{
    config::PlayerConfig,
    objects::animations,
    systems::{
        attack::Weapon, health::Health, AnimationController, Collider, ColliderObjectType,
        MyCollisionWorld, Player,
    },
    util::load_spritesheet,
};
use amethyst::{
    core::{math::Vector2, Transform},
    ecs::Entity,
    prelude::*,
    renderer::SpriteRender,
};
use std::collections::HashMap;

/// Will create the player with all its component. Should
/// be called once :D
///
/// position is the Spawn position of the player
pub fn create_player(world: &mut World, position: Transform) -> Entity {
    let sprite_sheet = load_spritesheet("Rogue", world);
    let sprite_render = SpriteRender {
        sprite_sheet,
        sprite_number: 0,
    };

    let player_config: PlayerConfig = *world
        .get_mut()
        .expect("World should have the player config resource");

    // SETUP ANIMATION
    // --------------------------------------------------------
    // Basic animation for the player
    // down left right up
    let walking_animations = animations::get_walking_animations();
    let mut animation_controller = AnimationController {
        animations: HashMap::new(),
        current_animation: None,
    };
    animation_controller.animations.extend(walking_animations);

    // SETUP COLLIDER
    // --------------------
    let collider = {
        let collision_world = world
            .get_mut::<MyCollisionWorld>()
            .expect("World should have `MyCollisionWorld` as a resource.");
        Collider::new_rect(
            Vector2::new(0.0, 0.0),
            16.0,
            16.0,
            &mut collision_world.world,
            ColliderObjectType::Player,
            None,
            None,
        )
    };

    // Now create with components
    // -------------------------
    let entity = world
        .create_entity()
        .with(position)
        .with(Player::default())
        .with(sprite_render)
        .with(animation_controller)
        .with(collider.clone())
        .with(Health::new(player_config.health))
        .with(Weapon::default())
        .build();

    let collision_world = world.get_mut::<MyCollisionWorld>().unwrap();
    collider.set_entity(&mut collision_world.world, entity);

    entity
}
