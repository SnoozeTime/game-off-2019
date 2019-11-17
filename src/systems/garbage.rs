//! Cleanup entities with their collider.
//! Deleting an entity requires to also delete its collider if attached.
//! This involves multiple systems so to avoid cluttering the systems,
//! entity deletion will be mostly done here when receiving events.
//!
//!
use crate::{
    event::AppEvent,
    systems::{Collider, MyCollisionWorld},
    util::delete_entity_with_collider,
};
use amethyst::{
    core::{
        shrev::{EventChannel, ReaderId},
        SystemDesc,
    },
    derive::SystemDesc,
    ecs::{Entities, Entity, Read, ReadStorage, System, SystemData, World, Write},
};
#[allow(unused_imports)]
use log::error;

#[derive(SystemDesc)]
#[system_desc(name(GarbageSystemDesc))]
pub struct GarbageSystem {
    #[system_desc(event_channel_reader)]
    reader_id: ReaderId<AppEvent>,
}

impl GarbageSystem {
    pub fn new(reader_id: ReaderId<AppEvent>) -> Self {
        Self { reader_id }
    }

    /// will destroy the entity and its colliders.
    fn boum(
        &self,
        entity: Entity,
        entities: &Entities,
        collision_world: &mut MyCollisionWorld,
        colliders: &ReadStorage<Collider>,
    ) {
        delete_entity_with_collider(entity, colliders, entities, &mut collision_world.world);
    }
}

impl<'s> System<'s> for GarbageSystem {
    type SystemData = (
        Entities<'s>,
        Write<'s, MyCollisionWorld>,
        Read<'s, EventChannel<AppEvent>>,
        ReadStorage<'s, Collider>,
    );

    fn run(&mut self, (entities, mut collision_world, events, colliders): Self::SystemData) {
        for ev in events.read(&mut self.reader_id) {
            match ev {
                AppEvent::EnemyDied(e) => {
                    self.boum(*e, &entities, &mut collision_world, &colliders)
                }
                _ => (),
            }
        }
    }
}
