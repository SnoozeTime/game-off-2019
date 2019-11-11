//! Game state is the main state of the game. (where the player actually plays)
use crate::tilemap;
use crate::util::delete_hierarchy;
use crate::z_layers::*;
use crate::{
    event::{AppEvent, MyEvent},
    objects::{enemy::EnemySpawner, player::create_player},
    systems::PlayerResource,
};
use amethyst::{
    core::transform::Transform,
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    renderer::{
        debug_drawing::{DebugLines, DebugLinesComponent, DebugLinesParams},
        Camera,
    },
    winit::VirtualKeyCode,
};
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

        *world.write_resource() = BulletSpawner::init(world);
        *world.write_resource() = EnemySpawner::init(world);

        // Setup debug lines as a resource
        world.insert(DebugLines::new());
        // Configure width of lines. Optional step
        world.insert(DebugLinesParams { line_width: 2.0 });
        world.register::<DebugLinesComponent>();

        // Activate the gameplay systems.
        *world.write_resource() = RuntimeSystemState::Running;

        let tilemap = tilemap::Tilemap::load("arena.tmx", world);
        let player_spawn = tilemap.player_spawn.clone();
        world.insert(tilemap);

        let player = create_player(
            world,
            player_spawn.unwrap_or_else(|| {
                let mut transform = Transform::default();
                transform.set_translation_xyz(50.0, 50.0, CHARACTERS_LAYER);
                transform
            }),
        );

        *world.write_resource() = PlayerResource {
            player: Some(player),
        };

        //add_bullet(world);
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
        let entities = Vec::from(data.world.get_mut::<tilemap::Tilemap>().unwrap().entities());

        if let Err(e) = data.world.delete_entities(&entities) {
            error!("{:?}", e);
        }
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
