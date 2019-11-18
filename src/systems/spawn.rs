//! Will manage enemy spawn :)
//! This is done on event. Should spawn an enemy at one of the spawn locations.
//!
//!
use crate::event::AppEvent;
use amethyst::{
    core::math::Vector2,
    core::{
        shrev::{EventChannel, ReaderId},
        SystemDesc,
    },
    derive::SystemDesc,
    ecs::{Component, Read, System, SystemData, VecStorage, World},
};
use log::info;

/// A spawn location is a place on the world where enemies will spawn.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct SpawnLocation {
    location: Vector2<f32>,
}

#[derive(SystemDesc)]
#[system_desc(name(SpawnSystemDesc))]
pub struct SpawnSystem {
    #[system_desc(event_channel_reader)]
    reader_id: ReaderId<AppEvent>,
}

impl SpawnSystem {
    pub fn new(reader_id: ReaderId<AppEvent>) -> Self {
        Self { reader_id }
    }
}

impl<'s> System<'s> for SpawnSystem {
    type SystemData = (Read<'s, EventChannel<AppEvent>>);

    fn run(&mut self, events: Self::SystemData) {
        // only one waves component.
        for ev in events.read(&mut self.reader_id) {
            match ev {
                AppEvent::SpawnEnemy(x) => info!("Spawn {} enemies", x),
                _ => (),
            }
        }
    }
}
