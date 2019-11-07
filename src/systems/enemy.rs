//! System that control the enemies
//!
use amethyst::{
    core::{SystemDesc, Transform},
    derive::SystemDesc,
    ecs::{
        Component, Join, NullStorage, Read, ReadStorage, System, SystemData, World, WriteStorage,
    },
};

use crate::systems::PlayerResource;

#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Enemy;

#[derive(SystemDesc)]
pub struct EnemySystem;

impl<'s> System<'s> for EnemySystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Enemy>,
        Read<'s, PlayerResource>,
    );

    fn run(&mut self, (mut transforms, enemies, player): Self::SystemData) {
        if let Some(e) = player.player {
            let player_transform = transforms
                .get(e)
                .expect("player should have a transform")
                .clone();
            let player_vec = player_transform.translation();

            for (t, _enemy) in (&mut transforms, &enemies).join() {
                let enemy_vec = t.translation();

                let d = (player_vec - enemy_vec).normalize();

                t.prepend_translation_x(1.2 * d.x);
                t.prepend_translation_y(1.2 * d.y);
            }
        }
    }
}
