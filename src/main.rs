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
    event::{MyEvent, MyEventReader},
    states,
    systems::{
        AnimationSystem, BulletSystem, DialogSystem, EnemySystem, PlayerSystem, WalkableSystem,
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

    let display_config_path = app_root.join("config").join("display.ron");
    let binding_path = app_root.join("config").join("bindings.ron");
    let input_bundle =
        InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;
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
        .with(
            WalkableSystem.pausable(states::RuntimeSystemState::Running),
            "walkable_system",
            &[],
        );

    let assets_dir = app_root.join("assets");
    let application = CoreApplication::<_, MyEvent, MyEventReader>::build(
        assets_dir,
        states::GameState::default(),
    )?
    //.with_resource(states::RuntimeSystemState::default())
    .build(game_data);

    application?.run();
    Ok(())
}
