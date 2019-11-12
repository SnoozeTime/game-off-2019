//! Manage different sort of collision
use crate::systems::{Player, PlayerStatus};
use crate::{event::AppEvent, util::delete_entity_with_collider};
use amethyst::{
    core::{
        math::{zero, Isometry2, Vector2},
        shrev::EventChannel,
        SystemDesc, Transform,
    },
    derive::SystemDesc,
    ecs::{
        Component, DenseVecStorage, Entities, Entity, Join, NullStorage, Read, ReadStorage, System,
        SystemData, World, Write, WriteStorage,
    },
};
use ncollide2d::{
    bounding_volume::{self, bounding_volume::BoundingVolume},
    pipeline::{
        narrow_phase::ContactEvent,
        object::{CollisionGroups, CollisionObjectSlabHandle, GeometricQueryType},
        world::CollisionWorld,
    },
    shape::{Cuboid, ShapeHandle},
};

use log::{debug, warn};

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

/// Determine what kind of object the collider is attached to. This is useful
/// when resolving collisions.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ColliderObjectType {
    Bullet,
    Player,
    Wall,
    Enemy,
    None,
}

impl ColliderObjectType {
    /// Collision group are used to tell what collides with what.
    /// For example, a bullet shot by an enemy should not collide with
    /// other enemies.
    pub fn get_collider_group(&self) -> usize {
        match *self {
            ColliderObjectType::Bullet => 1,
            ColliderObjectType::Player => 2,
            ColliderObjectType::Wall => 3,
            ColliderObjectType::Enemy => 4,
            ColliderObjectType::None => 5,
        }
    }
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
pub struct Collider {
    pub handle: CollisionObjectSlabHandle,
}

impl Collider {
    /// Create a new collider that has a rectangle shape. If entity is None, the entity can be
    /// associated as a second step using `set_entity`
    pub fn new_rect(
        position: Vector2<f32>,
        w: f32,
        h: f32,
        collision_world: &mut CollisionWorld<f32, ColliderData>,
        collider_type: ColliderObjectType,
        entity: Option<Entity>,
    ) -> Self {
        let rect = ShapeHandle::new(Cuboid::new(Vector2::new(w / 2.0, h / 2.0)));
        let position = Isometry2::new(position, zero());
        let mut group = CollisionGroups::new();
        group.set_membership(&[collider_type.get_collider_group()]);

        let contacts_query = GeometricQueryType::Contacts(0.0, 0.0);

        let (handle, _) = collision_world.add(
            position,
            rect,
            group,
            contacts_query,
            ColliderData {
                entity,
                ty: collider_type,
            },
        );
        Collider { handle }
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
        ReadStorage<'s, Collider>,
        ReadStorage<'s, Walkable>,
        Read<'s, MyCollisionWorld>,
    );

    fn run(
        &mut self,
        (mut players, colliders2, walkable_areas, collision_world): Self::SystemData,
    ) {
        for (player, collider) in (&mut players, &colliders2).join() {
            // If player is not walking, ignore the rest
            if let PlayerStatus::Walking = player.state {
                let mut player_in_area = false;

                let cob = collision_world
                    .world
                    .collision_object(collider.handle)
                    .unwrap();
                let aabb = bounding_volume::aabb(cob.shape().as_ref(), cob.position());
                for (_area, area_collider) in (&walkable_areas, &colliders2).join() {
                    let aob = collision_world
                        .world
                        .collision_object(area_collider.handle)
                        .unwrap();
                    let area_aabb = bounding_volume::aabb(aob.shape().as_ref(), aob.position());

                    if area_aabb.intersects(&aabb) {
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
        ReadStorage<'s, Collider>,
        ReadStorage<'s, Transform>,
        Write<'s, MyCollisionWorld>,
        Entities<'s>,
        Write<'s, EventChannel<AppEvent>>,
    );

    fn run(
        &mut self,
        (colliders, transforms, mut collision_world, entities, mut channel): Self::SystemData,
    ) {
        // handle here all the collision events.
        let mut to_remove = vec![];
        for event in collision_world.world.contact_events() {
            let mut to_remove_from_ev =
                self.handle_contact_event(&collision_world.world, event, &mut channel);
            to_remove.append(&mut to_remove_from_ev);
        }
        // now update all the positions and update the world.
        self.update_obj_positions(&colliders, &transforms, &mut collision_world);

        // Remove stuff to be destructed...
        for entity in to_remove {
            delete_entity_with_collider(entity, &colliders, &entities, &mut collision_world.world);
        }

        collision_world.world.update();
    }
}

impl CollisionSystem {
    /// Will update the collision object positiosn from the transform attached to the
    /// same entity.
    fn update_obj_positions(
        &self,
        colliders: &ReadStorage<Collider>,
        transforms: &ReadStorage<Transform>,
        collision_world: &mut Write<MyCollisionWorld>,
    ) {
        for (collider, transform) in (colliders, transforms).join() {
            if let Some(obj) = collision_world.world.get_mut(collider.handle) {
                let translation_xy = transform.translation().xy();
                let angle = transform.rotation().angle();
                let isometry = Isometry2::new(translation_xy, angle);
                obj.set_position(isometry);
            } else {
                warn!("Cannot find collision object for collider");
            }
        }
    }

    fn handle_contact_event(
        &self,
        world: &CollisionWorld<f32, ColliderData>,
        event: &ContactEvent<CollisionObjectSlabHandle>,
        channel: &mut Write<EventChannel<AppEvent>>,
    ) -> Vec<Entity> {
        let mut to_remove = vec![];
        if let &ContactEvent::Started(collider1, collider2) = event {
            let obj1 = world.collision_object(collider1).unwrap();
            let obj2 = world.collision_object(collider2).unwrap();

            match (obj1.data().ty, obj2.data().ty) {
                (ColliderObjectType::Player, ColliderObjectType::Bullet) => {
                    debug!("Object 1 (Player) collided with object 2 (Bullet)");
                    to_remove.push(
                        obj2.data()
                            .entity
                            .expect("Bullet should have an entity in its data"),
                    );
                    //entities.delete(obj2.data().ty);
                }
                (ColliderObjectType::Bullet, ColliderObjectType::Player) => {
                    debug!("Object 1 (Bullet) collided with object 2 (Player)");
                    to_remove.push(
                        obj1.data()
                            .entity
                            .expect("Bullet should have an entity in its data"),
                    );
                }
                (ColliderObjectType::Bullet, ColliderObjectType::Wall) => {
                    debug!("Bullet (1) collided with wall (2)");
                    to_remove.push(
                        obj1.data()
                            .entity
                            .expect("Bullet should have an entity in its data"),
                    );
                }
                (ColliderObjectType::Wall, ColliderObjectType::Bullet) => {
                    debug!("Bullet (1) collided with wall (1)");
                    to_remove.push(
                        obj2.data()
                            .entity
                            .expect("Bullet should have an entity in its data"),
                    );
                }
                _ => (),
            }

            channel.single_write(AppEvent::EntityHit(
                obj1.data()
                    .entity
                    .expect("obj1 collider should have an entity in its data"),
            ));
            channel.single_write(AppEvent::EntityHit(
                obj2.data()
                    .entity
                    .expect("obj2 collider should have an entity in its data"),
            ));
        }

        to_remove
    }
}
