// These imports are required for the #[derive(EventReader)] code to build
use crate::systems::schedule::ScheduledEvent;
use amethyst::core::{
    ecs::{Entity, Read, SystemData, World},
    shrev::{EventChannel, ReaderId},
    EventReader,
};
use amethyst::derive::EventReader;
use amethyst::{ui::UiEvent, winit::Event};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum AppEvent {
    /// Start a new dialog
    /// and_then would be an event to send once the dialog is over. It will be propagated to the
    /// DialogOver event and sent by the Game state (not the DialogState) when resuming.
    NewDialog {
        dialog: Vec<String>,
        and_then: Option<Arc<ScheduledEvent>>,
    },
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

    /// Start the next wave
    NextWave,

    /// All the waves are finshed. Let's start new arena (or boss :D)
    NextArena,
}

#[derive(Debug, EventReader, Clone)]
#[reader(MyEventReader)]
pub enum MyEvent {
    Window(Event),
    Ui(UiEvent),
    App(AppEvent),
}
