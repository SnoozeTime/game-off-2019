//! Manage different sort of collision
use crate::systems::{Player, PlayerStatus};
use amethyst::{
    core::{
        math::{zero, Isometry2, Point2, Vector2},
        SystemDesc, Transform,
    },
    derive::SystemDesc,
    ecs::{
        Component, DenseVecStorage, Entity, Join, NullStorage, ReadStorage, System, SystemData,
        VecStorage, World, Write, WriteStorage,
    },
};
use ncollide2d::{
    bounding_volume::{bounding_volume::BoundingVolume, AABB},
    pipeline::{
        narrow_phase::ContactEvent,
        object::{CollisionGroups, CollisionObjectSlabHandle, GeometricQueryType},
        world::CollisionWorld,
    },
    shape::{Cuboid, ShapeHandle},
};

use log::debug;

pub struct MyCollisionWorld {
    pub world: CollisionWorld<f32, ColliderData>,
}

impl Default for MyCollisionWorld {
    fn default() -> Self {
        Self {
            world: CollisionWorld::new(0.02),
        }
    }
}

/// Collider attached to an entity. Player and enemies should have one.
/// Walkable area as well.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Collider {
    pub bounding_volume: AABB<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum ColliderObjectType {
    Bullet,
    Player,
    None,
}

/// Collider component will have its entity as data.
#[derive(Debug, Clone, Copy)]
pub struct ColliderData {
    entity: Option<Entity>,
    ty: ColliderObjectType,
}

impl Default for ColliderData {
    fn default() -> Self {
        Self {
            entity: None,
            ty: ColliderObjectType::None,
        }
    }
}

#[derive(Debug, Component, Copy, Clone)]
#[storage(DenseVecStorage)]
pub struct Collider2 {
    pub handle: CollisionObjectSlabHandle,
}

impl Collider2 {
    pub fn new_rect(
        position: Vector2<f32>,
        w: f32,
        h: f32,
        collision_world: &mut CollisionWorld<f32, ColliderData>,
        collider_type: ColliderObjectType,
    ) -> Self {
        let rect = ShapeHandle::new(Cuboid::new(Vector2::new(w / 2.0, h / 2.0)));
        let position = Isometry2::new(position, zero());
        let group = CollisionGroups::new();
        let contacts_query = GeometricQueryType::Contacts(0.0, 0.0);

        let (handle, _) = collision_world.add(
            position,
            rect,
            group,
            contacts_query,
            ColliderData {
                entity: None,
                ty: collider_type,
            },
        );
        Collider2 { handle }
    }

    /// After creating the entity, we need to give it back to the collision object.
    pub fn set_entity(
        &self,
        collision_world: &mut CollisionWorld<f32, ColliderData>,
        entity: Entity,
    ) {
        let obj = collision_world
            .get_mut(self.handle)
            .expect("World should still have the object");
        obj.data_mut().entity = Some(entity);
    }
}

#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Walkable;

/// Will detect whether the player can still walk. This is
/// determine by checking if the player collider is still in the
/// walkable areas.
#[derive(Default, Debug, SystemDesc)]
pub struct WalkableSystem;

impl<'s> System<'s> for WalkableSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Collider>,
        ReadStorage<'s, Walkable>,
    );

    fn run(&mut self, (mut players, transforms, colliders, walkable_areas): Self::SystemData) {
        for (player, t, collider) in (&mut players, &transforms, &colliders).join() {
            // If player is not walking, ignore the rest
            if let PlayerStatus::Walking = player.state {
                let mut player_in_area = false;
                for (_area, area_collider) in (&walkable_areas, &colliders).join() {
                    // Now let's do the AABB testing.
                    // TODO find a better way...
                    let min: Point2<f32> = collider.bounding_volume.mins() + t.translation().xy();
                    let max = collider.bounding_volume.maxs() + t.translation().xy();
                    let translated_collider = AABB::new(min, max);
                    if area_collider
                        .bounding_volume
                        .intersects(&translated_collider)
                    {
                        player_in_area = true;
                        break;
                    }
                }

                // Now we know whether the player is above ground. If it's not, then he should
                // fall and the game is over. I don't send game over event right now. Rather, the
                // player fall animation needs to be played first.
                if !player_in_area {
                    player.state = PlayerStatus::Falling {
                        falling_duration: 1.0,
                        elapsed_time: 0.0,
                    };
                }
            }
        }
    }
}

#[derive(Debug, SystemDesc)]
pub struct CollisionSystem;

impl<'s> System<'s> for CollisionSystem {
    type SystemData = (
        ReadStorage<'s, Collider2>,
        ReadStorage<'s, Transform>,
        Write<'s, MyCollisionWorld>,
    );

    fn run(&mut self, (colliders, transforms, mut collision_world): Self::SystemData) {
        // handle here all the collision events.
        for event in collision_world.world.contact_events() {
            self.handle_contact_event(&collision_world.world, event);
        }

        // now update all the positions and update the world.
        self.update_obj_positions(&colliders, &transforms, &mut collision_world);
        collision_world.world.update();
    }
}

impl CollisionSystem {
    /// Will update the collision object positiosn from the transform attached to the
    /// same entity.
    fn update_obj_positions(
        &self,
        colliders: &ReadStorage<Collider2>,
        transforms: &ReadStorage<Transform>,
        collision_world: &mut Write<MyCollisionWorld>,
    ) {
        for (collider, transform) in (colliders, transforms).join() {
            let obj = collision_world.world.get_mut(collider.handle).unwrap();
            let translation_xy = transform.translation().xy();
            let angle = transform.rotation().angle();
            let isometry = Isometry2::new(translation_xy, angle);
            obj.set_position(isometry);
        }
    }

    fn handle_contact_event(
        &self,
        world: &CollisionWorld<f32, ColliderData>,
        event: &ContactEvent<CollisionObjectSlabHandle>,
    ) {
        if let &ContactEvent::Started(collider1, collider2) = event {
            // NOTE: real-life applications would avoid this systematic allocation.
            let obj1 = world.collision_object(collider1).unwrap();
            let obj2 = world.collision_object(collider2).unwrap();

            match (obj1.data().ty, obj2.data().ty) {
                (ColliderObjectType::Player, ColliderObjectType::Bullet) => {
                    debug!("Object 1 (Player) collided with object 2 (Bullet)");
                }
                (ColliderObjectType::Bullet, ColliderObjectType::Player) => {
                    debug!("Object 1 (Bullet) collided with object 2 (Player)");
                }
                _ => (),
            }
        }
    }
}
