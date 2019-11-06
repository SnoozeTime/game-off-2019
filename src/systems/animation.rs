use amethyst::core::{timing::Time, SystemDesc};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{
    Component, DenseVecStorage, Join, Read, System, SystemData, World, WriteStorage,
};
use amethyst::renderer::SpriteRender;
use log::error;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Animation {
    pub sprite_indexes: Vec<usize>,
    // in seconds
    pub step_duration: f32,
    pub current_index: usize,
    // in seconds
    pub elapsed_time: f32,
}

impl Animation {
    pub fn new(sprite_indexes: Vec<usize>, step_duration: f32) -> Self {
        Self {
            sprite_indexes,
            step_duration,
            current_index: 0,
            elapsed_time: 0.0,
        }
    }
}

/// All Animations for an entity
#[derive(Debug, Component, Default)]
#[storage(DenseVecStorage)]
pub struct AnimationController {
    /// Animation will cycle through the sprites on its spritesheet
    pub animations: HashMap<String, Animation>,

    /// if set to something, will play the corresponding animation
    pub current_animation: Option<String>,
}

#[derive(SystemDesc)]
pub struct AnimationSystem;

impl<'s> System<'s> for AnimationSystem {
    type SystemData = (
        WriteStorage<'s, AnimationController>,
        WriteStorage<'s, SpriteRender>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut animations, mut sprites, time): Self::SystemData) {
        for (controller, sprite) in (&mut animations, &mut sprites).join() {
            if let Some(ref animation_name) = controller.current_animation {
                if let Some(ref mut animation) = controller.animations.get_mut(animation_name) {
                    sprite.sprite_number = animation.sprite_indexes[animation.current_index];

                    animation.elapsed_time += time.delta_seconds();
                    if animation.elapsed_time > animation.step_duration {
                        animation.elapsed_time = 0.0;
                        animation.current_index =
                            (animation.current_index + 1) % animation.sprite_indexes.len();
                    }
                } else {
                    error!("Cannot find animation with name = {}", animation_name);
                }
            }
        }
    }
}
