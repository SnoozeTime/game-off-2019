//! Control the bullets on the map. Bullet will have some collision detection
//! to check whether they hit anything.
//!
use amethyst::{
    assets::{Prefab, PrefabData, ProgressCounter},
    core::{math::Vector2, timing::Time, SystemDesc, Transform},
    derive::PrefabData,
    derive::SystemDesc,
    ecs::{
        Component, DenseVecStorage, Entity, Join, Read, ReadStorage, System, SystemData, World,
        WriteStorage,
    },
    error::Error,
    renderer::sprite::prefab::SpriteScenePrefab,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PrefabData)]
pub struct BulletPrefab {
    sprite_scene: SpriteScenePrefab,
    bullet: Bullet,
    transform: Transform,
}

/// A basic bullet. Goes straight in a line with a given speed.
#[derive(Debug, Clone, Component, PrefabData, Serialize, Deserialize)]
#[prefab(Component)]
#[storage(DenseVecStorage)]
pub struct Bullet {
    /// Speed in unit/sec
    pub speed: f32,

    /// Where it is headed,
    pub direction: Vector2<f32>,
}

#[derive(SystemDesc)]
pub struct BulletSystem;

impl<'s> System<'s> for BulletSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Bullet>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, bullets, time): Self::SystemData) {
        for (bullet, t) in (&bullets, &mut transforms).join() {
            let delta_mvt = bullet.direction * bullet.speed * time.delta_seconds();

            // alternative, could set the transform rotation to face the shooting direction.
            // and use the move along method
            t.prepend_translation_x(delta_mvt.x);
            t.prepend_translation_y(delta_mvt.y);
        }
    }
}
