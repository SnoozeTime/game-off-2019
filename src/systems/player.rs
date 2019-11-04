use amethyst::core::{
    math::{Point3, Vector2},
    SystemDesc, Transform,
};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{
    Component, Entity, Join, NullStorage, Read, ReadStorage, System, SystemData, World,
    WriteStorage,
};
use amethyst::input::{InputHandler, StringBindings};
use nalgebra::{zero, Isometry2};
use ncollide2d::query::{Ray, RayCast};
use std::cmp::Ordering;

use crate::components::Obstacle;
use crate::systems::AnimationController;

/// Player entity is added as a resource :) This is to get it for
/// enemies that will need to follow the player
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerResource {
    pub player: Option<Entity>,
}

/// Attached only to the player. Act as a tag to
/// get it from the controller systems, or from
#[derive(Debug, Component, Default)]
#[storage(NullStorage)]
pub struct Player;

#[derive(SystemDesc)]
pub struct PlayerSystem;

impl<'s> System<'s> for PlayerSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        WriteStorage<'s, AnimationController>,
        ReadStorage<'s, Obstacle>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(
        &mut self,
        (mut transforms, players, mut animations, obstacles, input): Self::SystemData,
    ) {
        for (_player, transform, animation) in (&players, &mut transforms, &mut animations).join() {
            // idle state.
            animation.current_animation = None;
            let movement_x = input.axis_value("x");
            if let Some(mv_amount) = movement_x {
                if mv_amount > 0.0 && self.can_move_right(transform, &obstacles) {
                    let scaled_amount = 1.2 * mv_amount as f32;
                    transform.prepend_translation_x(scaled_amount);
                    animation.current_animation = Some("walk_right".to_string());
                } else if mv_amount < 0.0 && self.can_move_left(transform, &obstacles) {
                    let scaled_amount = 1.2 * mv_amount as f32;
                    transform.prepend_translation_x(scaled_amount);
                    animation.current_animation = Some("walk_left".to_string());
                }
            }

            let movement_y = input.axis_value("y");
            if let Some(mv_amount) = movement_y {
                if mv_amount > 0.0 && self.can_move_up(transform, &obstacles) {
                    let scaled_amount = 1.2 * mv_amount as f32;
                    transform.prepend_translation_y(scaled_amount);
                    animation.current_animation = Some("walk_up".to_string());
                } else if mv_amount < 0.0 && self.can_move_down(transform, &obstacles) {
                    let scaled_amount = 1.2 * mv_amount as f32;
                    transform.prepend_translation_y(scaled_amount);
                    animation.current_animation = Some("walk_down".to_string());
                }
            }
        }
    }
}

impl PlayerSystem {
    fn can_move_up(&self, t: &mut Transform, obstacles: &ReadStorage<Obstacle>) -> bool {
        let ray_direction = Vector2::<f32>::y();
        self.can_move_direction(t, ray_direction, obstacles)
    }
    fn can_move_down(&self, t: &mut Transform, obstacles: &ReadStorage<Obstacle>) -> bool {
        let ray_direction = -Vector2::<f32>::y();
        self.can_move_direction(t, ray_direction, obstacles)
    }
    fn can_move_left(&self, t: &mut Transform, obstacles: &ReadStorage<Obstacle>) -> bool {
        let ray_direction = -Vector2::<f32>::x();
        self.can_move_direction(t, ray_direction, obstacles)
    }
    fn can_move_right(&self, t: &mut Transform, obstacles: &ReadStorage<Obstacle>) -> bool {
        let ray_direction = Vector2::<f32>::x();
        self.can_move_direction(t, ray_direction, obstacles)
    }

    fn can_move_direction(
        &self,
        t: &mut Transform,
        direction: Vector2<f32>,
        obstacles: &ReadStorage<Obstacle>,
    ) -> bool {
        let player_origin = t
            .isometry()
            .transform_point(&Point3::new(0.0, 0.0, 0.0))
            .xy();
        let ray = Ray::new(player_origin, direction);
        let identity = Isometry2::new(zero(), zero());

        let cast = obstacles
            .join()
            .filter_map(|obs| obs.aabb.toi_with_ray(&identity, &ray, false))
            .min_by(|a: &f32, b: &f32| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        if let Some(toi) = cast {
            toi > 4.0
        } else {
            true
        }
    }
}
