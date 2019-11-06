//! Manage different sort of collision
use crate::systems::{Player, PlayerStatus};
use amethyst::{
    core::{math::Point2, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::{
        Component, Join, NullStorage, ReadStorage, System, SystemData, VecStorage, World,
        WriteStorage,
    },
};
use ncollide2d::bounding_volume::{bounding_volume::BoundingVolume, AABB};

/// Collider attached to an entity. Player and enemies should have one.
/// Walkable area as well.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Collider {
    pub bounding_volume: AABB<f32>,
}

#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct Walkable;

/// Will detect whether the player can still walk. This is
/// determine by checking if the player collider is still in the
/// walkable areas.
#[derive(Default, Debug, SystemDesc)]
pub struct WalkableSystem;

impl<'s> System<'s> for WalkableSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Collider>,
        ReadStorage<'s, Walkable>,
    );

    fn run(&mut self, (mut players, transforms, colliders, walkable_areas): Self::SystemData) {
        for (player, t, collider) in (&mut players, &transforms, &colliders).join() {
            // If player is not walking, ignore the rest
            if let PlayerStatus::Walking = player.state {
                let mut player_in_area = false;
                for (_area, area_collider) in (&walkable_areas, &colliders).join() {
                    // Now let's do the AABB testing.
                    // TODO find a better way...
                    let min: Point2<f32> = collider.bounding_volume.mins() + t.translation().xy();
                    let max = collider.bounding_volume.maxs() + t.translation().xy();
                    let translated_collider = AABB::new(min, max);
                    if area_collider
                        .bounding_volume
                        .intersects(&translated_collider)
                    {
                        player_in_area = true;
                        break;
                    }
                }

                // Now we know whether the player is above ground. If it's not, then he should
                // fall and the game is over. I don't send game over event right now. Rather, the
                // player fall animation needs to be played first.
                if !player_in_area {
                    player.state = PlayerStatus::Falling {
                        falling_duration: 1.0,
                        elapsed_time: 0.0,
                    };
                }
            }
        }
    }
}
