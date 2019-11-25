//! Common animations.
//!
use crate::systems::Animation;
use std::collections::HashMap;

/// Basic 4-direction walk animation.
pub fn get_walking_animations() -> HashMap<String, Animation> {
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
    let mut animations = HashMap::new();
    animations.insert("walk_down".to_string(), down_animation);
    animations.insert("walk_left".to_string(), left_animation);
    animations.insert("walk_right".to_string(), right_animation);
    animations.insert("walk_up".to_string(), up_animation);

    animations
}
pub fn get_enemy_simplest_animation() -> HashMap<String, Animation> {
    let down_animation = Animation {
        sprite_indexes: vec![0],
        current_index: 0,
        step_duration: 10.0,
        elapsed_time: 0.0,
    };
    let left_animation = Animation {
        sprite_indexes: vec![0],
        current_index: 0,
        step_duration: 10.0,
        elapsed_time: 0.0,
    };
    let right_animation = Animation {
        sprite_indexes: vec![0],
        current_index: 0,
        step_duration: 10.3,
        elapsed_time: 0.0,
    };
    let up_animation = Animation {
        sprite_indexes: vec![2],
        current_index: 0,
        step_duration: 10.3,
        elapsed_time: 0.0,
    };

    let shoot_animation = Animation {
        sprite_indexes: vec![1],
        current_index: 0,
        step_duration: 3.0,
        elapsed_time: 0.0,
    };
    let mut animations = HashMap::new();
    animations.insert("walk_down".to_string(), down_animation);
    animations.insert("walk_left".to_string(), left_animation);
    animations.insert("walk_right".to_string(), right_animation);
    animations.insert("walk_up".to_string(), up_animation);
    animations.insert("shoot".to_string(), shoot_animation);

    animations
}
