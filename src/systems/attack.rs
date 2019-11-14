//! Systems to manage the player attacks...
//! Player will be looking in the direction of the mouse. It will attack on mouse click

use amethyst::{
    core::{math::Vector3, timing::Time, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::{
        Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, SystemData, VecStorage,
        World, Write, WriteStorage,
    },
    input::{InputHandler, StringBindings},
    winit::MouseButton,
};

use crate::systems::{BulletSpawner, MyCollisionWorld, Player};

use log::{debug, error};
#[derive(Debug, Default, SystemDesc)]
pub struct AttackSystem;

#[derive(Debug, Clone, Copy, Component)]
#[storage(VecStorage)]
pub struct Weapon {
    /// Time before the enemy can shoot.
    time_before_shooting: f32,
    /// Number of sec the enemy will wait before shooting next.
    reload_time: f32,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            time_before_shooting: 0.0,
            reload_time: 1.0,
        }
    }
}

impl<'s> System<'s> for AttackSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Weapon>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Time>,
        // This is necessary to spawn bullets.
        Entities<'s>,
        Read<'s, LazyUpdate>,
        Read<'s, BulletSpawner>,
        Write<'s, MyCollisionWorld>,
    );

    fn run(
        &mut self,
        (
            players,
            transforms,
            mut weapon,
            input,
            time,
            entities,
            updater,
            bullet_spawner,
            mut collision_world,
        ): Self::SystemData,
    ) {
        // get the only player, transform tuple
        if let Some((_player, transform, weapon)) =
            (&players, &transforms, &mut weapon).join().next()
        {
            // update time before shooting.
            weapon.time_before_shooting =
                0.0f32.max(weapon.time_before_shooting - time.delta_seconds());

            if input.mouse_button_is_down(MouseButton::Left) {
                debug!(
                    "Mouse click at {:?} (player is at {:?}",
                    input.mouse_position(),
                    transform.translation()
                );

                if let Some((x, y)) = input.mouse_position() {
                    if weapon.time_before_shooting <= 0.0 {
                        let d = (Vector3::new(x, y, 0.0) - transform.translation())
                            .xy()
                            .normalize();
                        if let Err(e) = bullet_spawner.spawn_player_bullet(
                            &entities,
                            &updater,
                            &mut collision_world,
                            0,
                            *transform.translation(),
                            d,
                            100.0,
                        ) {
                            error!("Error while spawning player bullet = {:?}", e);
                        }
                        weapon.time_before_shooting = weapon.reload_time;
                    }
                }
            }
        }
    }
}
