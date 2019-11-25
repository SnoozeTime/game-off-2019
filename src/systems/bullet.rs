//! Control the bullets on the map. Bullet will have some collision detection
//! to check whether they hit anything.
//!
use crate::{
    config::BulletConfig,
    error::{GameError, GameResult},
    systems::{Collider, ColliderObjectType, MyCollisionWorld},
    util::load_spritesheet,
};

use amethyst::{
    assets::Handle,
    core::{
        math::{Rotation2, Vector2, Vector3},
        timing::Time,
        SystemDesc, Transform,
    },
    derive::SystemDesc,
    ecs::{
        Component, DenseVecStorage, Entities, Join, LazyUpdate, Read, System, SystemData, World,
        WriteStorage,
    },
    prelude::*,
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

    /// Bullet path (isometry applied to the direction at each update, so make sure it is small :D)
    pub path_mod: Option<Rotation2<f32>>,
}

/// Define behavior of a bullet.
pub enum BulletBehavior {
    /// Bullet that goes straight
    Simple { speed: f32, direction: Vector2<f32> },

    /// Bullet that rotations with a path
    Rotating {
        speed: f32,
        direction: Vector2<f32>,
        rotation: Rotation2<f32>,
    },
}

impl BulletBehavior {
    /// What to apply at each frame.
    pub fn apply(&mut self, bullet_transform: &mut Transform, time_delta: f32) {
        match *self {
            BulletBehavior::Simple {
                ref speed,
                ref direction,
            } => {
                let delta_mvt = *direction * *speed * time_delta;
                bullet_transform.prepend_translation_x(delta_mvt.x);
                bullet_transform.prepend_translation_y(delta_mvt.y);
            }
            BulletBehavior::Rotating {
                ref speed,
                ref mut direction,
                ref rotation,
            } => {
                let delta_mvt = *direction * *speed * time_delta;
                bullet_transform.prepend_translation_x(delta_mvt.x);
                bullet_transform.prepend_translation_y(delta_mvt.y);
                *direction = *rotation * *direction;
            }
        }
    }
}

#[derive(SystemDesc)]
pub struct BulletSystem;

impl<'s> System<'s> for BulletSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Bullet>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, mut bullets, time): Self::SystemData) {
        for (bullet, t) in (&mut bullets, &mut transforms).join() {
            let delta_mvt = bullet.direction * bullet.speed * time.delta_seconds();
            //trace!("Will move bullet by {:?}", delta_mvt);

            // alternative, could set the transform rotation to face the shooting direction.
            // and use the move along method
            t.prepend_translation_x(delta_mvt.x);
            t.prepend_translation_y(delta_mvt.y);

            // If we have some rotation to the bullet (yeah that is realistic), we need to change
            // the direction know.
            if let Some(modification) = bullet.path_mod {
                bullet.direction = modification * bullet.direction;
            }
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
    bullet_config: BulletConfig,
}

impl BulletSpawner {
    /// Will load all the textures of bullet in memory (actually
    /// stores handles) :)
    pub fn init(world: &mut World) -> Self {
        let handle = load_spritesheet("bullet", world);
        let textures = vec![handle];
        Self {
            textures,
            bullet_config: *world.read_resource::<BulletConfig>(),
        }
    }

    /// Spawn a new bullet.
    /// Entities will create a new entity. Updater will add components to the entity.
    ///
    /// bullet_idx should be found as a constant usize in this module.
    pub fn spawn_enemy_bullet(
        &self,
        entities: &Entities,
        updater: &LazyUpdate,
        collision: &mut MyCollisionWorld,
        bullet_idx: usize,
        origin: Vector3<f32>,
        direction: Vector2<f32>,
        speed: f32,
    ) -> GameResult<()> {
        self.spawn_bullet(
            entities,
            updater,
            collision,
            bullet_idx,
            origin,
            direction,
            speed,
            &[ColliderObjectType::Player, ColliderObjectType::Wall],
        )
    }
    pub fn spawn_player_bullet(
        &self,
        entities: &Entities,
        updater: &LazyUpdate,
        collision: &mut MyCollisionWorld,
        bullet_idx: usize,
        origin: Vector3<f32>,
        direction: Vector2<f32>,
        speed: f32,
    ) -> GameResult<()> {
        self.spawn_bullet(
            entities,
            updater,
            collision,
            bullet_idx,
            origin,
            direction,
            speed,
            &[ColliderObjectType::Enemy, ColliderObjectType::Wall],
        )
    }

    fn spawn_bullet(
        &self,
        entities: &Entities,
        updater: &LazyUpdate,
        collision: &mut MyCollisionWorld,
        bullet_idx: usize,
        origin: Vector3<f32>,
        direction: Vector2<f32>,
        speed: f32,
        collide_with: &[ColliderObjectType],
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
                    Some(collide_with),
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
            let rot = Rotation2::new(0.01f32);
            updater.insert(
                bullet,
                Bullet {
                    speed,
                    direction,
                    path_mod: Some(rot),
                    //path_mod: Some(Isometry2::new(Vector2::new(0.0, 0.0), 0.1)),
                },
            );
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
