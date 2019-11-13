use amethyst::{
    core::{
        shrev::{EventChannel, ReaderId},
        timing::Time,
        SystemDesc,
    },
    derive::SystemDesc,
    ecs::{
        Component, Entity, Read, ReadStorage, System, SystemData, VecStorage, World, WriteStorage,
    },
};

use crate::{event::AppEvent, systems::Player};
use log::debug;
/// Health - when reach 0, the entity is removed.
/// Health is an integer. A hit will always remove an entire portion of health
#[derive(Debug, Clone, Copy, Component)]
#[storage(VecStorage)]
pub struct Health {
    /// Current value of health.
    current_health: i32,

    /// Maximum value of health.
    max_health: i32,
}

impl Health {
    /// Will create a new health component with a given maximum health
    pub fn new(max_health: i32) -> Self {
        Self {
            current_health: max_health,
            max_health,
        }
    }
}

#[derive(SystemDesc)]
#[system_desc(name(HealthSystemDesc))]
pub struct HealthSystem {
    #[system_desc(event_channel_reader)]
    reader_id: ReaderId<AppEvent>,
}

impl HealthSystem {
    pub fn new(reader_id: ReaderId<AppEvent>) -> Self {
        Self { reader_id }
    }

    /// Decrease the health of the entity that has been hit.
    /// If it reaches 0, it will delete the entity and also emit an event (GameOver if player...)
    fn process_hit(
        &self,
        entity: Entity,
        health_storage: &mut WriteStorage<Health>,
        players: &ReadStorage<Player>,
    ) {
        debug!("Process Hit for entity {:?}", entity);
        if let Some(ref mut h) = health_storage.get_mut(entity) {
            h.current_health -= 1;
            if h.current_health <= 0 {
                debug!("Entity died :(");

                // let's check if that is the player :)
                if let Some(_) = players.get(entity) {
                    debug!("Game over...");
                }
            }
        } else {
            debug!("Entity does not have health component...");
        }
    }
}

impl<'s> System<'s> for HealthSystem {
    type SystemData = (
        WriteStorage<'s, Health>,
        ReadStorage<'s, Player>,
        Read<'s, Time>,
        Read<'s, EventChannel<AppEvent>>,
    );

    fn run(&mut self, (mut healths, players, _time, events): Self::SystemData) {
        for ev in events.read(&mut self.reader_id) {
            if let AppEvent::EntityHit(e) = ev {
                // In that case, one entity has been hit by a bullet so let's check if it has
                // some health component.
                self.process_hit(*e, &mut healths, &players);
            }
        }
    }
}
