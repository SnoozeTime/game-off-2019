use crate::components;
use crate::components::RenderObj;
use quicksilver::{
    geom::Rectangle,
    graphics::{Color, Image},
};
use snafu::{ResultExt, Snafu};
use specs::prelude::*;
use std::fmt;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Snafu)]
pub enum TileError {
    #[snafu(display("The Tiled map does not have a tileset at index {}", tileset_index))]
    NoTilesetForIndex { tileset_index: u32 },

    #[snafu(display("Tileset format not supported: {}", reason))]
    NotSupported { reason: String },

    #[snafu(display("Cannot find tileset image"))]
    CannotOpenImage { source: std::io::Error },

    #[snafu(display("Cannot convert bytes to image"))]
    ImageError { source: quicksilver::Error },
}

/// Represent a tile on a tile map. Need to have the name of the tileset as
/// well as the index in the tilemap.
#[derive(Debug)]
pub struct Tile {
    tileset: String,
    tile_nb: u32,
}

impl Tile {
    pub fn new(tileset: String, tile_nb: u32) -> Self {
        Self { tileset, tile_nb }
    }

    /// True if should display the tile. Tile with an index of 0
    /// are not to be displayed according to Tiled format.
    pub fn should_display(&self) -> bool {
        self.tile_nb != 0
    }
}

/// Metadata and asset for the tileset :)
pub struct Tileset {
    width: u32,
    height: u32,
    name: String,
    tile_width: u32,
    tile_height: u32,
    tile_per_row: u32,
    pub image_data: Image,
}

impl fmt::Debug for Tileset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Tileset: width {:?}, height: {:?}, name: {:?}, tile_size: {:?}, tile_per_row: {:?}",
            self.width,
            self.height,
            self.name,
            (self.tile_width, self.tile_height),
            self.tile_per_row
        )
    }
}

impl Tileset {
    /// Will load the tileset from the `Tiled` Map structure.
    ///
    /// Quicksilver will panic if cannot find Backend (graphic)...
    pub fn load(tiled_map: &tiled::Map, tileset_index: u32) -> Result<Self, TileError> {
        if let Some(tileset) = tiled_map.get_tileset_by_gid(tileset_index) {
            //tileset.tile_height
            if let None = tileset.images.get(0) {
                return Err(TileError::NotSupported {
                    reason: "No image for tileset".to_string(),
                });
            }

            let image = &tileset.images[0];
            let mut image_file = File::open(&image.source).context(CannotOpenImage {})?;
            let mut img_content = Vec::new();
            image_file
                .read_to_end(&mut img_content)
                .context(CannotOpenImage {})?;
            let image_data = Image::from_bytes(&img_content).context(ImageError {})?;

            let tile_per_row = image.width as u32 / tileset.tile_width as u32;
            Ok(Self {
                width: image.width as u32,
                height: image.height as u32,
                name: tileset.name.clone(),
                tile_width: tileset.tile_width,
                tile_height: tileset.tile_height,
                image_data,
                tile_per_row,
            })
        } else {
            Err(TileError::NoTilesetForIndex { tileset_index })
        }
    }

    pub fn get_tile_image(&self, tile: &Tile) -> Image {
        // x + y*width = index
        // x = index % width
        assert!(tile.tile_nb > 0);
        let x: u32 = (tile.tile_nb - 1) % self.tile_per_row;
        let y = (tile.tile_nb - x) / self.tile_per_row;

        self.image_data.subimage(Rectangle::new(
            (x * self.tile_width, y * self.tile_height),
            (self.tile_width, self.tile_height),
        ))
    }
}

/// Will create all the entities corresponding to a tile map.
pub fn create_entities_from_map(
    world: &mut World,
    map: &tiled::Map,
    player: Entity,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    // First the tile map.
    for (layer_nb, layer) in map.layers.iter().enumerate() {
        for (j, row) in layer.tiles.iter().enumerate() {
            for (i, tile_nb) in row.iter().enumerate() {
                let entity = world
                    .create_entity()
                    .with(components::Transform {
                        position: (i as u32 * 32, j as u32 * 32).into(),
                        rotation: 0.0,
                        scale: (1, 1).into(),
                    })
                    .with(components::Render {
                        layer_nb: layer_nb as u8,
                        to_render: RenderObj::Tile(Tile::new(String::new(), *tile_nb)),
                    })
                    .build();
                entities.push(entity);
            }
        }
    }

    // Now add the colliders :)
    let colliders = map
        .object_groups
        .iter()
        .filter(|&g| g.name == String::from("colliders"))
        .nth(0);
    if let Some(ref group) = colliders {
        for obj in &group.objects {
            if let tiled::ObjectShape::Rect { width, height, .. } = obj.shape {
                let r = Rectangle::new((obj.x, obj.y), (width, height));
                world
                    .create_entity()
                    .with(components::Obstacle {
                        aabb: r.into_aabb(),
                    })
                    .with(components::Transform {
                        position: (obj.x, obj.y).into(),
                        rotation: 0.0,
                        scale: (1, 1).into(),
                    })
                    .with(components::Render {
                        layer_nb: 87,
                        to_render: RenderObj::Rect(
                            Rectangle::new_sized((width, height)),
                            Color::BLUE,
                        ),
                    })
                    .build();
            }
        }
    }

    // Then enemies
    if let Some(ref group) = map
        .object_groups
        .iter()
        .filter(|&g| g.name == String::from("enemies"))
        .nth(0)
    {
        for obj in &group.objects {
            world
                .create_entity()
                .with(components::Transform {
                    position: (obj.x, obj.y).into(),
                    rotation: 0.0,
                    scale: (1, 1).into(),
                })
                .with(components::Enemy {
                    rotation_speed: 20.0,
                    alert: components::AlertStatus::Normal,
                    detection_angle: 45.0,
                    detection_distance: 150.0,
                    player: Some(player),
                })
                .with(components::Render {
                    to_render: RenderObj::Rect(Rectangle::new((-16, -16), (32, 32)), Color::GREEN),
                    layer_nb: 2,
                })
                .build();
        }
    }

    entities
}
