//! Game state is the main state of the game. (where the player actually plays)
use crate::tilemap;
use crate::util::{delete_hierarchy, load_spritesheet};
use crate::z_layers::*;
use crate::{
    event::{AppEvent, MyEvent},
    systems::{
        Animation, AnimationController, Bullet, Collider, Collider2, ColliderObjectType, Enemy,
        MyCollisionWorld, Player, PlayerResource,
    },
};
use amethyst::{
    assets::Handle,
    core::{
        math::{Point2, Vector2},
        transform::Transform,
    },
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    renderer::{
        debug_drawing::{DebugLines, DebugLinesComponent, DebugLinesParams},
        Camera, SpriteRender, SpriteSheet,
    },
    winit::VirtualKeyCode,
};
use log::{debug, error, info};
use ncollide2d::bounding_volume::AABB;
use std::collections::HashMap;

use super::{MyTrans, RuntimeSystemState, ARENA_HEIGHT, ARENA_WIDTH};

#[derive(Default, Debug)]
pub struct GameState {
    ui_handle: Option<Entity>,
}

impl State<GameData<'static, 'static>, MyEvent> for GameState {
    /// Called when state is starting.
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        world.register::<Player>();
        world.register::<Enemy>();

        // Setup debug lines as a resource
        world.insert(DebugLines::new());
        // Configure width of lines. Optional step
        world.insert(DebugLinesParams { line_width: 2.0 });
        world.register::<DebugLinesComponent>();

        // Activate the gameplay systems.
        *world.write_resource() = RuntimeSystemState::Running;

        let sprite_sheet = load_spritesheet("Rogue", world);
        let tilemap = tilemap::Tilemap::load("arena.tmx", world);
        let player_spawn = tilemap.player_spawn.clone();
        world.insert(tilemap);

        //        let events = world.get_mut::<EventChannel<AppEvent>>().unwrap();
        //        events.single_write(AppEvent::NewDialog(vec![
        //            "First".to_string(),
        //            "second".to_string(),
        //            "third".to_string(),
        //        ]));

        //self.ui_handle =
        //    Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/welcome.ron", ())));
        let player = initialize_player(
            world,
            player_spawn.unwrap_or_else(|| {
                let mut transform = Transform::default();
                transform.set_translation_xyz(50.0, 50.0, CHARACTERS_LAYER);
                transform
            }),
            sprite_sheet,
        );

        *world.write_resource() = PlayerResource {
            player: Some(player),
        };

        add_bullet(world);
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

/// Will place the player at the start of the level.
fn initialize_player(
    world: &mut World,
    transform: Transform,
    sprite_sheet: Handle<SpriteSheet>,
) -> Entity {
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };

    // Basic animation for the player
    // down left right up
    let down_animation = Animation {
        sprite_indexes: vec![0, 1, 2, 3],
        current_index: 0,
        step_duration: 0.3,
        elapsed_time: 0.0,
    };
    let left_animation = Animation {
        sprite_indexes: vec![4, 5, 6, 7],
        current_index: 0,
        step_duration: 0.3,
        elapsed_time: 0.0,
    };
    let right_animation = Animation {
        sprite_indexes: vec![8, 9, 10, 11],
        current_index: 0,
        step_duration: 0.3,
        elapsed_time: 0.0,
    };
    let up_animation = Animation {
        sprite_indexes: vec![12, 13, 14, 15],
        current_index: 0,
        step_duration: 0.3,
        elapsed_time: 0.0,
    };

    let mut animation_controller = AnimationController {
        animations: HashMap::new(),
        current_animation: None,
    };
    animation_controller
        .animations
        .insert("walk_down".to_string(), down_animation);
    animation_controller
        .animations
        .insert("walk_left".to_string(), left_animation);
    animation_controller
        .animations
        .insert("walk_right".to_string(), right_animation);
    animation_controller
        .animations
        .insert("walk_up".to_string(), up_animation);

    let collider = Collider {
        bounding_volume: AABB::new(Point2::new(0.0, 0.0), Point2::new(16.0, 16.0)),
    };

    let collider2 = {
        let collision_world = world.get_mut::<MyCollisionWorld>().unwrap();
        Collider2::new_rect(
            Vector2::new(0.0, 0.0),
            16.0,
            16.0,
            &mut collision_world.world,
            ColliderObjectType::Player,
        )
    };

    let entity = world
        .create_entity()
        .with(transform)
        .with(Player::default())
        .with(collider)
        .with(sprite_render)
        .with(animation_controller)
        .with(collider2.clone())
        .build();

    let collision_world = world.get_mut::<MyCollisionWorld>().unwrap();
    collider2.set_entity(&mut collision_world.world, entity);

    entity
}

fn add_bullet(world: &mut World) {
    let h = load_spritesheet("bullet", world);
    let mut t = Transform::default();
    t.append_translation_xyz(0.0, 0.0, 32.);

    let collider2 = {
        let collision_world = world.get_mut::<MyCollisionWorld>().unwrap();
        Collider2::new_rect(
            Vector2::new(0.0, 0.0),
            8.0,
            8.0,
            &mut collision_world.world,
            ColliderObjectType::Bullet,
        )
    };

    let entity = world
        .create_entity()
        .with(t)
        .with(Bullet {
            speed: 100.,
            direction: Vector2::new(1., 1.),
        })
        .with(SpriteRender {
            sprite_sheet: h,
            sprite_number: 0,
        })
        .with(collider2.clone())
        .build();

    let collision_world = world.get_mut::<MyCollisionWorld>().unwrap();
    collider2.set_entity(&mut collision_world.world, entity);
}
