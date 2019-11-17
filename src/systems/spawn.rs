//! Will manage enemy spawn :)
//! This is done on event. Should spawn an enemy at one of the spawn locations.
//!
//!
use amethyst::{
    core::math::Vector2,
    ecs::{Component, VecStorage},
};

/// A spawn location is a place on the world where enemies will spawn.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct SpawnLocation {
    location: Vector2<f32>,
}
