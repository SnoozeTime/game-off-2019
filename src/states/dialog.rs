use crate::event::{AppEvent, MyEvent};
use crate::util::delete_hierarchy;
use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    winit::VirtualKeyCode,
};
use log::{debug, info};

use super::{MyTrans, RuntimeSystemState};
use crate::systems::create_dialog;

/// Dialog state will handle a dialog. It is to be pushed onto the gameplay state and it will
/// freeze all the game systems.
#[derive(Debug, Default)]
pub struct DialogState {
    dialog_handle: Option<Entity>,
    dialog: Vec<String>,
}

impl DialogState {
    pub fn new(dialog: Vec<String>) -> Self {
        Self {
            dialog,
            dialog_handle: None,
        }
    }
}

impl State<GameData<'static, 'static>, MyEvent> for DialogState {
    fn on_start(&mut self, data: StateData<GameData>) {
        debug!("Start dialog state");
        let world = data.world;
        // deactivate the gameplay systems.
        {
            let mut state = world.write_resource::<RuntimeSystemState>();
            *state = RuntimeSystemState::Paused;
        }

        self.dialog_handle = Some(create_dialog(world, self.dialog.clone()));
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        debug!("Stop dialog state");
        if let Some(handler) = self.dialog_handle {
            let _res = delete_hierarchy(handler, data.world);
        }
        self.dialog_handle = None;
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
                if let AppEvent::DialogOver = e {
                    Trans::Pop
                } else {
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> MyTrans {
        data.data.update(&data.world);
        MyTrans::None
    }
}
