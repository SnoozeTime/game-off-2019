use crate::components::*;
use nalgebra::{zero, Isometry2};
use ncollide2d::query::{Ray, RayCast};
use quicksilver::geom;
use quicksilver::geom::Vector;
use quicksilver::graphics::Color;
use specs::{Join, ReadStorage, System, WriteStorage};
use std::cmp::Ordering;

const FIXED_TS: f32 = 1.0 / 60.0;

fn rad2deg(rad: f32) -> f32 {
    rad * 180.0 / std::f32::consts::PI
}

// ===================================================================
// Move the player
// -------------------------------------------------------------------
pub struct PlayerSystem;
impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        ReadStorage<'a, Player>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Obstacle>,
    );

    /// Will control the player. Also check for collisions on the way to stop movement
    fn run(&mut self, (players, mut transforms, obstacles): Self::SystemData) {
        for (player, t) in (&players, &mut transforms).join() {
            let can_move_up = self.can_move_up(t, &obstacles);
            if player.move_left {
                t.position.x -= 4.0;
            }

            if player.move_right {
                t.position.x += 4.0;
            }

            if player.move_up && can_move_up {
                t.position.y -= 4.0;
            }

            if player.move_down {
                t.position.y += 4.0;
            }
        }
    }
}

impl PlayerSystem {
    fn can_move_up(
        &self,
        player_transform: &mut Transform,
        obstacles: &ReadStorage<Obstacle>,
    ) -> bool {
        let ray_direction = -Vector::Y.into_vector();

        let ray = Ray::new(player_transform.position.into_point(), ray_direction);
        let identity = Isometry2::new(zero(), zero());

        let cast = obstacles
            .join()
            .filter_map(|obs| obs.aabb.toi_with_ray(&identity, &ray, false))
            .min_by(|a: &f32, b: &f32| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        if let Some(toi) = cast {
            toi > 4.0
        } else {
            true
        }
    }
}

// ===================================================================
// Move the enemy
// --------------------------------------------------------------------

pub struct EnemySystem;
impl<'a> System<'a> for EnemySystem {
    type SystemData = (
        ReadStorage<'a, Enemy>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Display>,
    );

    fn run(&mut self, (enemies, mut transforms, _): Self::SystemData) {
        for (transform, enemy) in (&mut transforms, &enemies).join() {
            transform.rotation += enemy.rotation_speed * FIXED_TS;
        }
    }
}

// ================================================================
// Detection system with whether enemy can see the player
// ----------------------------------------------------------------
pub struct DetectionSystem;
impl<'a> System<'a> for DetectionSystem {
    type SystemData = (
        WriteStorage<'a, Enemy>,
        ReadStorage<'a, Obstacle>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Render>, // TODO replace that with physic component
    );

    fn run(&mut self, (mut enemies, obstacles, transforms, mut renders): Self::SystemData) {
        for (enemy, enemy_transform) in (&mut enemies, &transforms).join() {
            // will send raycast from the enemy to the player. if cross
            // a wall, player should not be dtected.
            if let Some(player) = enemy.player {
                // TODO - can do it once. the player stays the same...
                let player_transform = transforms
                    .get(player)
                    .expect("Player should have a transform. That's weird otherwise");
                let player_display = renders
                    .get(player)
                    .expect("Player should have a display (for AABB)");

                // STEP 1. Is player in angle of vision?
                // ------------------------------------
                let ray_direction =
                    (player_transform.position - enemy_transform.position).normalize();
                let player_enemy_angle =
                    rad2deg(ray_direction.dot(enemy_transform.direction()).acos());

                if player_enemy_angle < enemy.detection_angle
                    && player_enemy_angle > -enemy.detection_angle
                {
                    // now, raycast to the player
                    let start = enemy_transform.position.into_point();
                    let ray = Ray::new(start, ray_direction.into_vector());
                    let identity = Isometry2::new(zero(), zero());
                    let player_aabb = player_display.get_transformed_aabb(
                        geom::Transform::translate(player_transform.position),
                    );

                    // Now, distance enemy to player.
                    let toi_player = player_aabb.toi_with_ray(&identity, &ray, false);
                    let cast = obstacles
                        .join()
                        .filter_map(|obs| obs.aabb.toi_with_ray(&identity, &ray, false))
                        .min_by(|a: &f32, b: &f32| a.partial_cmp(b).unwrap_or(Ordering::Equal));

                    if let Some(toi_player) = toi_player {
                        let line_of_sight;
                        if let Some(toi_region) = cast {
                            line_of_sight = toi_region > toi_player;
                        } else {
                            line_of_sight = true;
                        }

                        if line_of_sight {
                            if toi_player < enemy.detection_distance {
                                enemy.alert = AlertStatus::Alert;
                            } else if toi_player < enemy.detection_distance * 2.0 {
                                enemy.alert = AlertStatus::Curious;
                            } else {
                                enemy.alert = AlertStatus::Normal;
                            }
                        } else {
                            enemy.alert = AlertStatus::Normal;
                        }
                    } else {
                        enemy.alert = AlertStatus::Normal;
                    }
                } else {
                    enemy.alert = AlertStatus::Normal;
                }
            }
        }

        // Now change the enemy colors.
        for (enemy, display) in (&enemies, &mut renders).join() {
            if let RenderObj::Rect(_, ref mut c) = display.to_render {
                match enemy.alert {
                    AlertStatus::Normal => *c = Color::GREEN,
                    AlertStatus::Curious => *c = Color::YELLOW,
                    AlertStatus::Alert => *c = Color::RED,
                }
            }
        }
    }
}
