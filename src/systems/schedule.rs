//! Schedule events by creating an entity with a Schedule component.
//! The entity is destroyed after the event fires. (Maybe later we can setup some repeating
//! events...)

use crate::event::AppEvent;
use amethyst::{
    core::{shrev::EventChannel, timing::Time, SystemDesc},
    derive::SystemDesc,
    ecs::{
        Component, Entities, Join, Read, System, SystemData, VecStorage, World, Write, WriteStorage,
    },
};
#[allow(unused_imports)]
use log::{debug, error};

/// Component to schedule an event in a fixed amount of time.
///
/// Should be the only component on one entity. In particular, should absolutely not be attached to
/// an entity with a collider because the scheduler system will delete the entity once the event
/// has been fired.
#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct ScheduledEvent {
    /// Time it takes until the event is sent (in seconds)
    pub timeout: f32,

    /// Event to fire.
    pub event: AppEvent,
}

#[derive(SystemDesc)]
pub struct Scheduler;

impl<'s> System<'s> for Scheduler {
    type SystemData = (
        WriteStorage<'s, ScheduledEvent>,
        Read<'s, Time>,
        Entities<'s>,
        Write<'s, EventChannel<AppEvent>>,
    );

    fn run(&mut self, (mut scheduled_events, time, entities, mut channel): Self::SystemData) {
        let mut to_delete = vec![];
        for (scheduled_event, e) in (&mut scheduled_events, &entities).join() {
            scheduled_event.timeout -= time.delta_seconds();
            if scheduled_event.timeout <= 0.0 {
                // FIRE OFFFFFF
                debug!("Will send {:?}", scheduled_event.event);
                channel.single_write(scheduled_event.event.clone());
                to_delete.push(e);
            }
        }

        to_delete.iter().for_each(|e| {
            if let Err(e) = entities.delete(*e) {
                error!("{:?}", e);
            }
        });
    }
}
