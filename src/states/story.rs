//! Story state. at the beginning of the game, it will explain the background of the
//! story before getting started with the tutorial.
use amethyst::{
    core::shrev::EventChannel,
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down, is_mouse_button_down},
    prelude::*,
    ui::UiCreator,
    winit::{MouseButton, VirtualKeyCode},
};
use log::info;

use crate::event::{AppEvent, MyEvent};
use crate::util::delete_hierarchy;

#[derive(Default, Debug)]
pub struct StoryState {
    ui_handle: Option<Entity>,
}

use super::MyTrans;

impl State<GameData<'static, 'static>, MyEvent> for StoryState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        let events = world.get_mut::<EventChannel<AppEvent>>().unwrap();
        events.single_write(AppEvent::NewDialog {
            dialog: vec!["First".to_string(), "second".to_string()],
            and_then: None,
        });
        self.ui_handle =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/story.ron", ())));
    }

    fn on_resume(&mut self, data: StateData<GameData>) {
        let world = data.world;
        *world.write_resource() = super::RuntimeSystemState::Running;
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(handler) = self.ui_handle {
            delete_hierarchy(handler, data.world).expect("Failed to remove WelcomeScreen");
        }
        self.ui_handle = None;
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: MyEvent) -> MyTrans {
        match &event {
            MyEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_mouse_button_down(&event, MouseButton::Left) {
                    info!("[Trans::Switch] Switching to MainMenu!");
                    Trans::Switch(Box::new(crate::states::GameState::default()))
                } else {
                    Trans::None
                }
            }
            MyEvent::App(e) => {
                if let AppEvent::NewDialog {
                    dialog: sentences, ..
                } = e
                {
                    Trans::Push(Box::new(crate::states::DialogState::new(sentences.clone())))
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
