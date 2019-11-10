use amethyst::ecs::{Component, VecStorage};

/// Health - when reach 0, the entity is removed.
/// Health is an integer. A hit will always remove an entire portion of health
#[derive(Debug, Clone, Copy, Component)]
#[storage(VecStorage)]
pub struct Health {
    /// Current value of health.
    current_health: i32,

    /// Maximum value of health.
    max_health: i32,
}

impl Health {
    /// Will create a new health component with a given maximum health
    pub fn new(max_health: i32) -> Self {
        Self {
            current_health: max_health,
            max_health,
        }
    }
}
