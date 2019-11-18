// These imports are required for the #[derive(EventReader)] code to build
use amethyst::core::{
    ecs::{Entity, Read, SystemData, World},
    shrev::{EventChannel, ReaderId},
    EventReader,
};
use amethyst::derive::EventReader;
use amethyst::{ui::UiEvent, winit::Event};

#[derive(Clone, Debug)]
pub enum AppEvent {
    /// Start a new dialog
    NewDialog(Vec<String>),
    /// Generated when the current dialog is finished
    DialogOver,
    /// Generated when the game is finished (player loses)
    GameOver,
    /// Generated when an entity is hit by a bullet.
    EntityHit(Entity),
    /// Generated when an entity has died
    EnemyDied(Entity),
    /// Spawn a certain number of enemies
    SpawnEnemy(i32),
}

#[derive(Debug, EventReader, Clone)]
#[reader(MyEventReader)]
pub enum MyEvent {
    Window(Event),
    Ui(UiEvent),
    App(AppEvent),
}
