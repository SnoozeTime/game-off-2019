use amethyst::core::{
    math::{Point3, Vector2, Vector3},
    shrev::EventChannel,
    timing::Time,
    SystemDesc, Transform,
};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{
    Component, Entity, Join, Read, ReadStorage, System, SystemData, VecStorage, World, Write,
    WriteStorage,
};
use amethyst::input::{InputHandler, StringBindings};
use log::{info, trace};
use nalgebra::{zero, Isometry2};
use ncollide2d::query::{Ray, RayCast};
use std::cmp::Ordering;

use crate::{components::Obstacle, event::AppEvent, systems::AnimationController};

const FALL_ROT_SPEED: f32 = 45.0;

fn get_scale(falling_duration: f32, elapsed_time: f32) -> f32 {
    ((falling_duration - elapsed_time) / falling_duration).max(0.0)
}

/// Player entity is added as a resource :) This is to get it for
/// enemies that will need to follow the player
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerResource {
    pub player: Option<Entity>,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerStatus {
    /// The player is above ground ! and can move, shoot normally
    Walking,

    /// The player is falling from the arena. The game is over but this state is needed
    /// for transition (animation...)
    Falling {
        falling_duration: f32,
        elapsed_time: f32,
    },

    /// Don't do anything, Just wait for game over.
    GameOver,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        PlayerStatus::Walking
    }
}

/// Attached only to the player. Act as a tag to
/// get it from the controller systems, or from
#[derive(Debug, Component, Default)]
#[storage(VecStorage)]
pub struct Player {
    pub state: PlayerStatus,
}

#[derive(SystemDesc)]
pub struct PlayerSystem;

impl<'s> System<'s> for PlayerSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, AnimationController>,
        ReadStorage<'s, Obstacle>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Time>,
        Write<'s, EventChannel<AppEvent>>,
    );

    fn run(
        &mut self,
        (mut transforms, mut players, mut animations, obstacles, input, time, mut event): Self::SystemData,
    ) {
        for (player, transform, animation) in
            (&mut players, &mut transforms, &mut animations).join()
        {
            // idle state.
            //
            match player.state {
                PlayerStatus::Walking => self.player_walk(transform, animation, &input, &obstacles),
                PlayerStatus::Falling { .. } => self.player_fall(
                    player,
                    transform,
                    animation,
                    time.delta_seconds(),
                    &mut event,
                ),
                _ => (),
            }
        }
    }
}

impl PlayerSystem {
    fn player_fall(
        &self,
        player: &mut Player,
        transform: &mut Transform,
        animation: &mut AnimationController,
        time_delta: f32,
        event: &mut Write<EventChannel<AppEvent>>,
    ) {
        // No animation, the only thing we do is to modify the transform to make it like the
        // character falls.
        animation.current_animation = None;

        let mut has_fallen = false;
        if let PlayerStatus::Falling {
            ref falling_duration,
            ref mut elapsed_time,
        } = player.state
        {
            *elapsed_time = *elapsed_time + time_delta;

            let scale = get_scale(*falling_duration, *elapsed_time);
            transform.set_scale(Vector3::new(scale, scale, scale));
            transform.prepend_rotation_z_axis(FALL_ROT_SPEED * time_delta);
            trace!("SCALE {}", scale);
            trace!("elapsed {} / {}", elapsed_time, falling_duration);
            has_fallen = *elapsed_time > *falling_duration;
        }

        if has_fallen {
            info!("Player has fallen, game over");
            player.state = PlayerStatus::GameOver;
            event.single_write(AppEvent::GameOver);
        }
    }

    /// Player walks with the input axis. Raycast to not walk in walls
    fn player_walk(
        &self,
        transform: &mut Transform,
        animation: &mut AnimationController,
        input: &InputHandler<StringBindings>,
        obstacles: &ReadStorage<Obstacle>,
    ) {
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
