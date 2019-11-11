//! Helpers to create the player entity...
//!
use crate::{
    config::PlayerConfig,
    systems::{
        health::Health, Animation, AnimationController, Collider, ColliderObjectType,
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
    let down_animation = Animation {
        sprite_indexes: vec![0, 1, 2, 3],
        current_index: 0,
        step_duration: 0.3,
        elapsed_time: 0.0,
    };
    let left_animation = Animation {
        sprite_indexes: vec![4, 5, 6, 7],
        current_index: 0,
        step_duration: 0.3,
        elapsed_time: 0.0,
    };
    let right_animation = Animation {
        sprite_indexes: vec![8, 9, 10, 11],
        current_index: 0,
        step_duration: 0.3,
        elapsed_time: 0.0,
    };
    let up_animation = Animation {
        sprite_indexes: vec![12, 13, 14, 15],
        current_index: 0,
        step_duration: 0.3,
        elapsed_time: 0.0,
    };

    let mut animation_controller = AnimationController {
        animations: HashMap::new(),
        current_animation: None,
    };
    animation_controller
        .animations
        .insert("walk_down".to_string(), down_animation);
    animation_controller
        .animations
        .insert("walk_left".to_string(), left_animation);
    animation_controller
        .animations
        .insert("walk_right".to_string(), right_animation);
    animation_controller
        .animations
        .insert("walk_up".to_string(), up_animation);

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
        //.with(Health::new(player_config.health))
        .build();

    let collision_world = world.get_mut::<MyCollisionWorld>().unwrap();
    collider.set_entity(&mut collision_world.world, entity);

    entity
}
