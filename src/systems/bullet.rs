//! Control the bullets on the map. Bullet will have some collision detection
//! to check whether they hit anything.
//!
use crate::{
    error::{GameError, GameResult},
    systems::{Collider, ColliderObjectType, MyCollisionWorld},
    util::load_spritesheet,
};

use amethyst::{
    assets::Handle,
    core::{
        math::{Vector2, Vector3},
        timing::Time,
        SystemDesc, Transform,
    },
    derive::SystemDesc,
    ecs::{
        Component, DenseVecStorage, Entities, Join, LazyUpdate, Read, ReadStorage, System,
        SystemData, World, WriteStorage,
    },
    renderer::{SpriteRender, SpriteSheet},
};

#[allow(unused_imports)]
use log::{debug, trace};
use serde::{Deserialize, Serialize};

/// A basic bullet. Goes straight in a line with a given speed.
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
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
            //trace!("Will move bullet by {:?}", delta_mvt);

            // alternative, could set the transform rotation to face the shooting direction.
            // and use the move along method
            t.prepend_translation_x(delta_mvt.x);
            t.prepend_translation_y(delta_mvt.y);
        }
    }
}

// ==================================================================
// Keep the bullet texture handles in the world's resources. This is
// setup at on_start of the game state
// ==================================================================

pub const SIMPLE_BULLET_IDX: usize = 0;

/// Used as a resource to spawn new bullets from a bullet ID and a direction and speed.
#[derive(Debug, Default)]
pub struct BulletSpawner {
    textures: Vec<Handle<SpriteSheet>>,
}

impl BulletSpawner {
    /// Will load all the textures of bullet in memory (actually
    /// stores handles) :)
    pub fn init(world: &mut World) -> Self {
        let handle = load_spritesheet("bullet", world);
        let textures = vec![handle];
        Self { textures }
    }

    /// Spawn a new bullet.
    /// Entities will create a new entity. Updater will add components to the entity.
    ///
    /// bullet_idx should be found as a constant usize in this module.
    pub fn spawn_bullet(
        &self,
        entities: &Entities,
        updater: &LazyUpdate,
        collision: &mut MyCollisionWorld,
        bullet_idx: usize,
        origin: Vector3<f32>,
        direction: Vector2<f32>,
        speed: f32,
    ) -> GameResult<()> {
        debug!(
            "Will spawn bullet {} from point {:?} with direction {:?} at speed {}",
            bullet_idx, origin, direction, speed
        );

        if let Some(handle) = self.textures.get(bullet_idx) {
            let bullet = entities.create();

            let collider = {
                let collider = Collider::new_rect(
                    Vector2::new(0.0, 0.0),
                    8.0,
                    8.0,
                    &mut collision.world,
                    ColliderObjectType::Bullet,
                    Some(bullet),
                );
                //collider.set_entddity(&mut collision.world, bullet);
                collider
            };

            let mut t = Transform::default();
            // don't want the bullet to fly to fast :D
            let direction = direction.normalize();
            t.prepend_translation(origin);
            t.set_translation_z(20.0);
            updater.insert(bullet, Bullet { speed, direction });
            updater.insert(
                bullet,
                SpriteRender {
                    sprite_sheet: handle.clone(),
                    sprite_number: 0,
                },
            );
            updater.insert(bullet, t);
            updater.insert(bullet, collider);
            Ok(())
        } else {
            Err(GameError::BulletNotFound { idx: bullet_idx })
        }
    }
}
