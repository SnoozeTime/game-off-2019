use crate::{
    event::{AppEvent, MyEvent},
    states::MyTrans,
    util::delete_hierarchy,
};
use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder},
    winit::VirtualKeyCode,
};
use log::info;

const RETRY_BUTTON_ID: &str = "retry";
const EXIT_TO_MAIN_MENU_BUTTON_ID: &str = "exit_to_main_menu";
const EXIT_BUTTON_ID: &str = "exit";

/// Just display a text and then propose to start again or go back to main menu
#[derive(Debug, Default)]
pub struct GameOverState {
    root: Option<Entity>,

    // Buttons entities are created on_start and destroy on_stop()
    retry_button: Option<Entity>,
    exit_to_main_menu_button: Option<Entity>,
    exit_button: Option<Entity>,
}

impl State<GameData<'static, 'static>, MyEvent> for GameOverState {
    fn on_start(&mut self, data: StateData<GameData>) {
        info!("Start gameover state");
        let world = data.world;
        self.root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/gameover.ron", ())));
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(handler) = self.root {
            delete_hierarchy(handler, data.world).expect("Failed to remove GameOverScreen");
        }
        self.root = None;
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
            MyEvent::Ui(UiEvent {
                event_type: UiEventType::Click,
                target,
            }) => {
                if Some(*target) == self.retry_button {
                    info!("[Trans::Switch] Switching to Game!");
                    Trans::Switch(Box::new(crate::states::GameState::default()))
                } else if Some(*target) == self.exit_button {
                    info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
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

    /// Will get the entities for the UI elements from UiFinder.
    fn update(&mut self, data: StateData<GameData>) -> MyTrans {
        data.data.update(&data.world);

        if self.retry_button.is_none()
            || self.exit_to_main_menu_button.is_none()
            || self.exit_button.is_none()
        {
            data.world.exec(|ui_finder: UiFinder<'_>| {
                self.retry_button = ui_finder.find(RETRY_BUTTON_ID);
                self.exit_button = ui_finder.find(EXIT_BUTTON_ID);
                self.exit_to_main_menu_button = ui_finder.find(EXIT_TO_MAIN_MENU_BUTTON_ID);
            });
        }

        MyTrans::None
    }
}
