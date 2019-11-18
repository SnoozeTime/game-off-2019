//! Game state is the main state of the game. (where the player actually plays)
use crate::tilemap;
use crate::util::delete_hierarchy;
use crate::z_layers::*;
use crate::{
    config::ArenaConfig,
    event::{AppEvent, MyEvent},
    objects::{enemy::EnemySpawner, player::create_player},
    systems::{wave, Bullet, Collider, MyCollisionWorld, PlayerResource},
    util::delete_entity_with_collider,
};
use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Entities, Entity, Join, Read, ReadStorage, Write},
    input::{is_close_requested, is_key_down},
    prelude::*,
    renderer::{
        debug_drawing::{DebugLines, DebugLinesComponent, DebugLinesParams},
        Camera,
    },
    utils::application_root_dir,
    winit::VirtualKeyCode,
};
#[allow(unused_imports)]
use log::{debug, error, info};

use super::{MyTrans, RuntimeSystemState, ARENA_HEIGHT, ARENA_WIDTH};
use crate::systems::BulletSpawner;
#[derive(Default, Debug)]
pub struct GameState {
    ui_handle: Option<Entity>,
}

impl State<GameData<'static, 'static>, MyEvent> for GameState {
    /// Called when state is starting.
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        debug!("Add Bullet spawner resource");
        let bullet_spawner = BulletSpawner::init(world);
        world.insert(bullet_spawner);
        let enemy_spawner = EnemySpawner::init(world);
        debug!("Add EnemySpawner resource");
        world.insert(enemy_spawner);

        // Setup debug lines as a resource
        world.insert(DebugLines::new());
        // Configure width of lines. Optional step
        world.insert(DebugLinesParams { line_width: 2.0 });
        world.register::<DebugLinesComponent>();

        // Activate the gameplay systems.
        *world.write_resource() = RuntimeSystemState::Running;

        debug!("Load tilemap");
        let tilemap = tilemap::Tilemap::load("arena.tmx", world);
        let player_spawn = tilemap.player_spawn.clone();
        world.insert(tilemap);

        debug!("Create player");
        let player = create_player(
            world,
            player_spawn.unwrap_or_else(|| {
                let mut transform = Transform::default();
                transform.set_translation_xyz(50.0, 50.0, CHARACTERS_LAYER);
                transform
            }),
        );

        world.insert(PlayerResource {
            player: Some(player),
        });

        debug!("Create wave");
        let app_root = application_root_dir().unwrap();
        let config_file = app_root.join("config").join("wave1.ron");
        let arena_config = ArenaConfig::load(&config_file);
        let waves = wave::Waves::from_config(arena_config);
        world.create_entity().with(waves).build();

        //add_bullet(world);
        debug!("Init camera");
        initialize_camera(world);
    }

    /// Activate the gameplay systems that might have been paused by other states (such as dialog)
    fn on_resume(&mut self, data: StateData<GameData>) {
        let world = data.world;
        *world.write_resource() = RuntimeSystemState::Running;
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(handler) = self.ui_handle {
            delete_hierarchy(handler, data.world).expect("Failed to remove WelcomeScreen");
        }

        data.world.exec(
            |(tilemap, mut collisions_world, entities, colliders, player, bullets): (
                Read<tilemap::Tilemap>,
                Write<MyCollisionWorld>,
                Entities,
                ReadStorage<Collider>,
                Read<PlayerResource>,
                ReadStorage<Bullet>,
            )| {
                for e in tilemap.entities().iter().cloned() {
                    delete_entity_with_collider(
                        e,
                        &colliders,
                        &entities,
                        &mut collisions_world.world,
                    );
                }
                if let Some(e) = player.player {
                    delete_entity_with_collider(
                        e,
                        &colliders,
                        &entities,
                        &mut collisions_world.world,
                    );
                }

                // remove in flight bullets
                for (_bullet, entity) in (&bullets, &entities).join() {
                    delete_entity_with_collider(
                        entity,
                        &colliders,
                        &entities,
                        &mut collisions_world.world,
                    );
                }
            },
        );
        self.ui_handle = None;
    }

    fn update(&mut self, data: StateData<GameData>) -> MyTrans {
        data.data.update(&data.world);
        MyTrans::None
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: MyEvent) -> MyTrans {
        match &event {
            MyEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            MyEvent::App(e) => {
                // In case the state receives an event for dialog, it will push a DialogState on
                // top of the stack to pause the gameplay systems.
                if let AppEvent::NewDialog(sentences) = e {
                    Trans::Push(Box::new(crate::states::DialogState::new(sentences.clone())))
                } else if let AppEvent::GameOver = e {
                    Trans::Switch(Box::new(crate::states::GameOverState::default()))
                } else {
                    debug!("{:?}", e);
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }
}

/// Set up camera. Orthographic projection to see an area of 100x100 units
fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 99.0);
    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build();
}
