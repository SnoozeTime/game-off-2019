use amethyst::{prelude::*, utils::application_root_dir};
use thief_engine::config::ArenaConfig;

fn main() {
    let app_root = application_root_dir().unwrap();
    let config_file = app_root.join("config").join("wave1.ron");
    let waves = ArenaConfig::load(&config_file);
    println!("{:?}", waves);
}
