use amethyst::{
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderDebugLines, RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::application_root_dir,
};
use thief_engine::{
    config::{BulletConfig, CameraConfig, EnemyConfig, PlayerConfig},
    event::{MyEvent, MyEventReader},
    states,
    systems::{
        attack, health, schedule, spawn, wave, AnimationSystem, BulletSystem, CollisionSystemDesc,
        DialogSystem, EnemySystem, MyCollisionWorld, PlayerSystem, WalkableSystem,
    },
};

use log::info;

fn configure_logger() {
    pretty_env_logger::init();
    info!("Logger is setup correctly");
}

fn main() -> amethyst::Result<()> {
    configure_logger();
    //amethyst::start_logger(Default::default());
    let app_root = application_root_dir()?;

    // Load all the config files
    // -----------------------------------------------------
    let config_path = app_root.join("config").join("player.ron");
    let arena_config_path = app_root.join("config").join("camera.ron");
    let enemy_config_path = app_root.join("config").join("enemy.ron");
    let bullet_config_path = app_root.join("config").join("bullet.ron");
    let player_config = PlayerConfig::load(&config_path);
    let arena_config = CameraConfig::load(&arena_config_path);
    let enemy_config = EnemyConfig::load(&enemy_config_path);
    let bullet_config = BulletConfig::load(&bullet_config_path);

    let display_config_path = app_root.join("config").join("display.ron");
    let binding_path = app_root.join("config").join("bindings.ron");
    let input_bundle =
        InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;

    // Collision world will keep track of all the collision objects. It will be
    // added as a resource of the amethyst application
    let collision_world = MyCollisionWorld::default();

    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderDebugLines::default())
                // Without this, all of our beautiful UI would not get drawn.
                // It will work, but we won't see a thing.
                .with_plugin(RenderUi::default()),
        )?
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        // this bundle allows us to 'find' the Buttons and other UI elements later on
        .with_bundle(UiBundle::<StringBindings>::new())?
        // --------------------------------
        .with(
            PlayerSystem.pausable(states::RuntimeSystemState::Running),
            "player_system",
            &["input_system"],
        )
        .with(AnimationSystem, "animation_system", &["player_system"])
        .with(DialogSystem, "dialog_system", &["input_system"])
        .with(
            EnemySystem.pausable(states::RuntimeSystemState::Running),
            "enemy_system",
            &[],
        )
        .with(
            BulletSystem.pausable(states::RuntimeSystemState::Running),
            "bullet_system",
            &[],
        )
        .with_system_desc(CollisionSystemDesc, "collision_system", &[])
        .with_system_desc(wave::WaveSystemDesc, "wave_system", &[])
        .with_system_desc(spawn::SpawnSystemDesc, "spawn_system", &[])
        .with(
            WalkableSystem.pausable(states::RuntimeSystemState::Running),
            "walkable_system",
            &["collision_system"],
        )
        .with_system_desc(
            health::HealthSystemDesc,
            "health_system",
            &["collision_system"],
        )
        .with(attack::AttackSystem, "attack_system", &["input_system"])
        .with(schedule::Scheduler, "scheduler", &[]);

    let assets_dir = app_root.join("assets");
    let application = CoreApplication::<_, MyEvent, MyEventReader>::build(
        assets_dir,
        states::GameOverState::default(),
    )?
    .with_resource(player_config)
    .with_resource(arena_config)
    .with_resource(enemy_config)
    .with_resource(bullet_config)
    .with_resource(collision_world)
    .build(game_data);

    application?.run();
    Ok(())
}
