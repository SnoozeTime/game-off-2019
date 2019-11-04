use ncollide2d::bounding_volume::AABB;
use specs::{Component, VecStorage};

// -------------------------------------------------------------------

/// Obstacles block the line of vision of enemies, so that the player
/// can go undetected.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Obstacle {
    pub aabb: AABB<f32>,
}
