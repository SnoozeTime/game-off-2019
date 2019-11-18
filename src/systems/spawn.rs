//! Will manage enemy spawn :)
//! This is done on event. Should spawn an enemy at one of the spawn locations.
//!
//!
use crate::{
    event::AppEvent,
    objects::enemy::EnemySpawner,
    systems::{enemy::EnemyType, MyCollisionWorld},
    z_layers::PROPS_LAYER,
};
use amethyst::{
    core::math::Vector2,
    core::{
        shrev::{EventChannel, ReaderId},
        SystemDesc, Transform,
    },
    derive::SystemDesc,
    ecs::{
        Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, SystemData, VecStorage,
        World, Write,
    },
};
use log::{error, info};
use rand::seq::SliceRandom;

/// A spawn location is a place on the world where enemies will spawn.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct SpawnLocation {
    pub location: Vector2<f32>,
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
    type SystemData = (
        ReadStorage<'s, SpawnLocation>,
        Read<'s, EventChannel<AppEvent>>,
        // For spawning enemies.
        Read<'s, EnemySpawner>,
        Entities<'s>,
        Read<'s, LazyUpdate>,
        Write<'s, MyCollisionWorld>,
    );

    fn run(
        &mut self,
        (locations, events, spawner, entities, updater, mut collision_world): Self::SystemData,
    ) {
        // only one waves component.
        for ev in events.read(&mut self.reader_id) {
            match ev {
                AppEvent::SpawnEnemy(x) => {
                    info!("Spawn {} enemies", x);
                    let mut rng = rand::thread_rng();
                    let locations = (&locations).join().collect::<Vec<_>>();
                    for _ in 0..*x {
                        let location = locations.choose(&mut rng);
                        if let Some(SpawnLocation { location }) = location {
                            println!("Will spawn at loc {:?}", location);
                            let mut t = Transform::default();
                            t.append_translation_xyz(location.x, location.y, PROPS_LAYER);
                            if let None = spawner.spawn_enemy(
                                &entities,
                                &updater,
                                &mut collision_world,
                                EnemyType::Simple,
                                t,
                            ) {
                                error!(
                                    "Could not find enemy {:?} in Spawner - Check init...",
                                    EnemyType::Simple
                                );
                            }
                        } else {
                            error!("Spawner cannot choose a location. Make sure there are some setup...");
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
