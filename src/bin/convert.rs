//! CLI utility
//!
//!
use image::image_dimensions;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde_derive::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "convert", about = "Create the spritesheet RON file")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(long)]
    tile_width: usize,

    #[structopt(long)]
    tile_height: usize,
}

#[derive(Debug, Serialize)]
struct Spritesheet {
    texture_width: usize,
    texture_height: usize,
    sprites: Vec<Sprite>,
}

#[derive(Debug, Serialize)]
struct Sprite {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

fn main() {
    let Opt {
        mut input,
        tile_width,
        tile_height,
    } = Opt::from_args();

    let (width, height) = image_dimensions(input.clone()).expect("LOL2");

    let width = width as usize;
    let height = height as usize;

    assert!(input.set_extension("ron"));
    // sanity check the dimensions.
    assert!(width % tile_width == 0);
    assert!(height % tile_height == 0);

    let mut spritesheet = Spritesheet {
        texture_width: width,
        texture_height: height,
        sprites: vec![],
    };

    for y in (0..height).step_by(tile_height) {
        for x in (0..width).step_by(tile_width) {
            spritesheet.sprites.push(Sprite {
                x,
                y,
                width: tile_width,
                height: tile_height,
            });
        }
    }

    let pretty = PrettyConfig::default();
    let s = to_string_pretty(&spritesheet, pretty).expect("Serialization failed");

    let mut fs = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(input)
        .expect("Cannot open file");

    fs.write(s.as_bytes()).unwrap();
}
