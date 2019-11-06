use crate::{
    event::{AppEvent, MyEvent},
    states::MyTrans,
    util::delete_hierarchy,
};
use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down, is_mouse_button_down},
    prelude::*,
    ui::UiCreator,
    winit::{MouseButton, VirtualKeyCode},
};
use log::info;

/// Just display a text and then propose to start again or go back to main menu
#[derive(Debug, Default)]
pub struct GameOverState {
    ui_handle: Option<Entity>,
}

impl State<GameData<'static, 'static>, MyEvent> for GameOverState {
    fn on_start(&mut self, data: StateData<GameData>) {
        info!("Start gameover state");
        let world = data.world;
        self.ui_handle =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/story.ron", ())));
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(handler) = self.ui_handle {
            delete_hierarchy(handler, data.world).expect("Failed to remove GameOverScreen");
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
                if let AppEvent::NewDialog(sentences) = e {
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
