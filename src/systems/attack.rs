//! Systems to manage the player attacks...
//! Player will be looking in the direction of the mouse. It will attack on mouse click

use amethyst::{
    core::{SystemDesc, Transform},
    derive::SystemDesc,
    ecs::{Join, Read, ReadStorage, System, SystemData, World},
    input::{InputHandler, StringBindings},
    winit::MouseButton,
};

use crate::systems::Player;

use log::debug;
#[derive(Debug, Default, SystemDesc)]
pub struct AttackSystem;

impl<'s> System<'s> for AttackSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Transform>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (players, transforms, input): Self::SystemData) {
        // get the only player, transform tuple
        if let Some((_player, transform)) = (&players, &transforms).join().next() {
            if input.mouse_button_is_down(MouseButton::Left) {
                debug!(
                    "Mouse click at {:?} (player is at {:?}",
                    input.mouse_position(),
                    transform.translation()
                );
            }
        }
    }
}
