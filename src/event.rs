// These imports are required for the #[derive(EventReader)] code to build
use amethyst::core::{
    ecs::{Read, SystemData, World},
    shrev::{EventChannel, ReaderId},
    EventReader,
};
use amethyst::derive::EventReader;
use amethyst::{ui::UiEvent, winit::Event};

#[derive(Clone, Debug)]
pub enum AppEvent {
    NewDialog(Vec<String>),
    DialogOver,
    GameOver,
}

#[derive(Debug, EventReader, Clone)]
#[reader(MyEventReader)]
pub enum MyEvent {
    Window(Event),
    Ui(UiEvent),
    App(AppEvent),
}
