//! Systems to manage the player attacks...
//! Player will be looking in the direction of the mouse. It will attack on mouse click

use amethyst::{
    core::{
        geometry::Plane,
        math::{Point2, Vector2},
        timing::Time,
        SystemDesc, Transform,
    },
    derive::SystemDesc,
    ecs::{
        Component, Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData,
        VecStorage, World, Write, WriteStorage,
    },
    input::{InputHandler, StringBindings},
    renderer::camera::{ActiveCamera, Camera},
    window::ScreenDimensions,
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
        // This is necessary for getting the mouse position projected on the camera view.
        ReadStorage<'s, Camera>,
        ReadExpect<'s, ScreenDimensions>,
        Read<'s, ActiveCamera>,
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
            cameras,
            screen_dimensions,
            active_camera,
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
                if let Some((x, y)) = input.mouse_position() {
                    let mut camera_join = (&cameras, &transforms).join();
                    if let Some((camera, camera_transform)) = active_camera
                        .entity
                        .and_then(|a| camera_join.get(a, &entities))
                        .or_else(|| camera_join.next())
                    {
                        // Project a ray from the camera to the 0z axis
                        let ray = camera.projection().screen_ray(
                            Point2::new(x, y),
                            Vector2::new(screen_dimensions.width(), screen_dimensions.height()),
                            camera_transform,
                        );
                        let distance = ray.intersect_plane(&Plane::with_z(0.0)).unwrap();
                        let mouse_world_position = ray.at_distance(distance);
                        debug!(
                            "Mouse click at {:?} (player is at {:?}",
                            mouse_world_position,
                            transform.translation()
                        );
                        if weapon.time_before_shooting <= 0.0 {
                            let d = mouse_world_position - transform.translation();
                            let d = Vector2::new(d.x, d.y).normalize();
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
}
