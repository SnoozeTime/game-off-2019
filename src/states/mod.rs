use crate::event::MyEvent;
use amethyst::prelude::*;
mod story;
pub use story::StoryState;

mod dialog;
pub use dialog::DialogState;

mod game;
pub use game::GameState;

pub const ARENA_WIDTH: f32 = 800.0;
pub const ARENA_HEIGHT: f32 = 600.0;

pub type MyTrans = Trans<GameData<'static, 'static>, MyEvent>;

/// Will determine whether some runtime systems should be running or paused.
/// For example, during dialog the player system should not run..._
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuntimeSystemState {
    Running,
    Paused,
}

impl std::default::Default for RuntimeSystemState {
    fn default() -> Self {
        RuntimeSystemState::Paused
    }
}
