//! System that control the enemies
//!
use amethyst::{
    core::{math::Vector3, timing::Time, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::{
        Component, Entities, Join, LazyUpdate, Read, System, SystemData, VecStorage, World, Write,
        WriteStorage,
    },
};

use crate::{
    config::EnemyConfig,
    systems::{AnimationController, BulletSpawner, MyCollisionWorld, PlayerResource},
};
#[allow(unused_imports)]
use log::{error, info};

/// Current status of the enemy. The behavior will depend on
/// what enemy type and not all enemies can be in all states.
#[derive(Debug, Clone, Copy)]
pub enum EnemyStatus {
    /// Walk.
    Walking,
    /// Shoot a bullet towards the player.
    Shooting,
    /// Wait a bit before resuming the walking.
    PostShooting,
    Dying,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum EnemyType {
    /// Most basic enemy. Follow and shoot.
    Simple,
}

#[derive(Debug, Clone, Copy, Component)]
#[storage(VecStorage)]
pub enum Enemy {
    Simple(SimpleEnemy),
    CreepyFirstBoss,
}

impl Enemy {
    pub fn from_config(_enemy_type: EnemyType, config: &EnemyConfig) -> Self {
        Enemy::Simple(SimpleEnemy {
            bullet_speed: config.simple_enemy.bullet_speed,
            walk_duration: config.simple_enemy.walk_duration,
            shoot_duration: config.simple_enemy.shoot_duration,
            ..SimpleEnemy::default()
        })
    }

    /// What to do every frame
    pub fn update(
        &mut self,
        delta_time: f32,
        t: &mut Transform,
        animations: &mut Option<&mut AnimationController>,

        player_vec: &Vector3<f32>,
        bullet_spawner: &Read<BulletSpawner>,
        entities: &Entities,
        updater: &Read<LazyUpdate>,
        collision: &mut Write<MyCollisionWorld>,
    ) {
        match *self {
            Enemy::Simple(ref mut simple_enemy) => simple_enemy.update(
                delta_time,
                t,
                animations,
                player_vec,
                bullet_spawner,
                entities,
                updater,
                collision,
            ),
            _ => (),
        }
    }
}

impl Default for Enemy {
    fn default() -> Self {
        Enemy::Simple(SimpleEnemy::default())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SimpleEnemy {
    /// How fast its bullet flies.
    bullet_speed: f32,

    /// how fast it walks.
    walk_speed: f32,

    /// what the enemy is currently doing
    state: EnemyStatus,

    /// State control
    /// How long have we been in the current state.
    current_state_duration: f32,

    /// How long does the enemy walk.
    walk_duration: f32,

    /// Shoot duration. How long does the enemy shoot? It is related to the shooting animation.
    shoot_duration: f32,
}

impl Default for SimpleEnemy {
    fn default() -> Self {
        Self {
            bullet_speed: 100.0,
            state: EnemyStatus::Walking,

            walk_speed: 0.2,
            current_state_duration: 0.0,
            walk_duration: 3.0,
            shoot_duration: 0.3,
        }
    }
}

impl SimpleEnemy {
    /// Simple enemy can be in three states. Either moving, shooting or dying.
    /// Each states has its own animations.
    /// Walking, it is just hovering towards the player.
    /// Shooting will take a few frames. First it opens the mouth and a bullet will come out of it.
    /// When the enemy has been killed, there is a death animation before the entity is removed.
    fn update(
        &mut self,
        delta_time: f32,
        t: &mut Transform,
        maybe_anim: &mut Option<&mut AnimationController>,
        player_vec: &Vector3<f32>,
        bullet_spawner: &Read<BulletSpawner>,
        entities: &Entities,
        updater: &Read<LazyUpdate>,
        collision: &mut Write<MyCollisionWorld>,
    ) {
        let enemy_vec = t.translation();

        let mut previous_animation = None;
        if let Some(ref mut anim) = maybe_anim {
            previous_animation = anim.current_animation.take();
        }
        let direction = player_vec - enemy_vec;

        //let dist = direction.norm();
        let d = direction.normalize();

        self.current_state_duration += delta_time;
        match self.state {
            EnemyStatus::Walking => {
                t.prepend_translation_x(self.walk_speed * d.x);
                t.prepend_translation_y(self.walk_speed * d.y);

                if let Some(ref mut anim) = maybe_anim {
                    if d.x < 0.0 {
                        anim.current_animation = Some("walk_left".to_string());
                    } else if d.x > 0.0 {
                        anim.current_animation = Some("walk_right".to_string());
                    } else if d.y > 0.0 {
                        anim.current_animation = Some("walk_up".to_string());
                    } else if d.y < 0.0 {
                        anim.current_animation = Some("walk_down".to_string());
                    }
                }

                // state transition if walked too long
                if self.current_state_duration >= self.walk_duration {
                    self.state = EnemyStatus::Shooting;
                    self.current_state_duration = 0.0;
                }
            }

            EnemyStatus::Shooting => {
                // shoot at the player :D
                if let Err(e) = bullet_spawner.spawn_enemy_bullet(
                    entities,
                    updater,
                    collision,
                    0,
                    *t.translation(),
                    direction.xy(),
                    self.bullet_speed,
                ) {
                    error!("Enemy cannot spawn bullet: {}", e);
                }

                // shoot animation is only visible when the enemy is facing down...
                if let Some(ref mut anim) = maybe_anim {
                    match previous_animation {
                        Some(ref x) if x.as_str() == "walk_up" => (),
                        _ => anim.current_animation = Some("shoot".to_string()),
                    }
                }

                self.current_state_duration = 0.0;
                self.state = EnemyStatus::PostShooting;
            }

            EnemyStatus::PostShooting => {
                if let Some(ref mut anim) = maybe_anim {
                    match previous_animation {
                        Some(ref x) if x.as_str() == "walk_up" => (),
                        _ => anim.current_animation = Some("shoot".to_string()),
                    }
                }

                if self.current_state_duration >= self.shoot_duration {
                    self.current_state_duration = 0.0;
                    self.state = EnemyStatus::Walking;
                }
            }
            _ => (),
        }
    }
}
#[derive(SystemDesc)]
pub struct EnemySystem;

impl<'s> System<'s> for EnemySystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Enemy>,
        Read<'s, PlayerResource>,
        Read<'s, Time>,
        Entities<'s>,
        Read<'s, LazyUpdate>,
        Read<'s, BulletSpawner>,
        Write<'s, MyCollisionWorld>,
        WriteStorage<'s, AnimationController>,
    );

    fn run(
        &mut self,
        (
            mut transforms,
            mut enemies,
            player,
            time,
            entities,
            updater,
            bullet_spawner,
            mut collision,
            mut animations,
        ): Self::SystemData,
    ) {
        if let Some(e) = player.player {
            if let Some(player_transform) = transforms.get(e).cloned() {
                let player_vec = player_transform.translation();

                for (t, enemy, e) in (&mut transforms, &mut enemies, &entities).join() {
                    enemy.update(
                        time.delta_seconds(),
                        t,
                        &mut animations.get_mut(e),
                        &player_vec,
                        &bullet_spawner,
                        &entities,
                        &updater,
                        &mut collision,
                    );
                }
            }
        }
        // info!("Processed data for {} enemies", i);
    }
}
