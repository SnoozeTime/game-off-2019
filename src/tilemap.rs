//! Module to create entities from a Tiled map.
//!
//!
use crate::components::Obstacle;
use crate::states::ARENA_HEIGHT;
use crate::{
    systems::{Animation, AnimationController, Enemy},
    util::load_spritesheet,
    z_layers::*,
};
use amethyst::utils::application_root_dir;
use amethyst::{
    assets::Handle,
    core::{math::geometry::Point2, transform::Transform},
    ecs::Entity,
    prelude::*,
    renderer::{debug_drawing::DebugLinesComponent, palette::Srgba, SpriteRender, SpriteSheet},
};
use ncollide2d::bounding_volume::AABB;
use std::path::PathBuf;
use tiled::parse_file;
use tracing::warn;

/// Contains all the tile entities and props
/// All contains metadata (player spawn position, map name...)
#[derive(Debug, Default)]
pub struct Tilemap {
    /// all the tiles (not props) of the tilemap
    pub all_tiles: Vec<Entity>,

    /// all colliders
    pub all_colliders: Vec<Entity>,

    /// all objects
    pub all_props: Vec<Entity>,

    //pub all_enemies: Vec<Entity>,
    /// Initial position for the player
    pub player_spawn: Option<Transform>,
}

impl Tilemap {
    /// Load the map from the tmx file.
    pub fn load(map_name: &str, world: &mut World) -> Tilemap {
        let mut tilemap = Tilemap::default();
        let app_root = application_root_dir().unwrap();
        let tilemap_path = app_root.join("assets").join("tilemap").join(map_name);

        let map = parse_file(&tilemap_path).unwrap();

        // When there are multiple tilesets, the index of the tile in the map does not
        // correspond to the index of the tile in the tileset. I need to remove an offset that is
        // stored as first gid.
        let mut first_gids = vec![];
        let mut tileset_names = vec![];
        for tileset in &map.tilesets {
            first_gids.push(tileset.first_gid);
            let tileset_name = PathBuf::from(tileset.images[0].source.clone());
            let tileset_name = tileset_name.file_stem().unwrap().to_str().unwrap();
            //tileset_path.pop()
            tileset_names.push(load_spritesheet(tileset_name, world));
        }

        first_gids.sort();

        for layer in &map.layers {
            let layer_name = layer.name.to_lowercase();
            for (y, row) in layer.tiles.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    if *tile != 0 {
                        let x = x as f32 * 16.0;
                        let y = ARENA_HEIGHT - (1.0 + y as f32) * 16.0;
                        let tid = choose_tileset(*tile, &first_gids);
                        let offset = first_gids[tid];
                        let real_tile_id = *tile - offset;

                        let mut transform = Transform::default();
                        let z_layer = match layer_name.as_str() {
                            "floor" => FLOOR_LAYER,
                            "walls" => WALLS_LAYER,
                            _ => 0.0,
                        };
                        transform.set_translation_xyz(x + 8.0, y + 8.0, z_layer);
                        tilemap.all_tiles.push(
                            world
                                .create_entity()
                                .with(SpriteRender {
                                    sprite_number: real_tile_id as usize,
                                    sprite_sheet: tileset_names[tid].clone(),
                                })
                                .with(transform)
                                .build(),
                        );
                    }
                }
            }
        }

        tilemap.load_colliders(&map, world);
        tilemap.load_props(&map, world, &first_gids, &tileset_names);
        tilemap.load_player_spawn(&map);
        tilemap.load_enemies(&map, world);

        tilemap
    }

    /// Player spawn is in its own object layer. There should be only one object
    /// which is a point.
    fn load_player_spawn(&mut self, map: &tiled::Map) {
        if let Some(ref group) = map
            .object_groups
            .iter()
            .filter(|&g| g.name == String::from("player"))
            .nth(0)
        {
            if let Some(ref player_spawn) = group.objects.iter().nth(0) {
                // Here
                let (x, y) = convert_tiled_xy(player_spawn.x, player_spawn.y);
                let mut t = Transform::default();
                t.set_translation_xyz(x, y, CHARACTERS_LAYER);
                self.player_spawn = Some(t);
            } else {
                warn!("No object in `player` layer, will use default spawn for player");
            }
        } else {
            warn!("No `player` layer in loaded tilemap");
        }
    }

    fn load_enemies(&mut self, map: &tiled::Map, world: &mut World) {
        if let Some(ref group) = map
            .object_groups
            .iter()
            .filter(|&g| g.name == String::from("enemy"))
            .nth(0)
        {
            let spritesheet = load_spritesheet("rectangle", world);
            for obj in &group.objects {
                let (x, y) = convert_tiled_xy(obj.x, obj.y);
                let mut transform = Transform::default();
                transform.set_translation_xyz(x + 8.0, y + 8.0, PROPS_LAYER);

                let sprite = SpriteRender {
                    sprite_sheet: spritesheet.clone(),
                    sprite_number: 0,
                };
                world
                    .create_entity()
                    .with(transform)
                    .with(Enemy)
                    .with(sprite)
                    .build();
            }
        }
    }

    /// Load the colliders from the map. They are attached as objects in an
    /// object layer called `colliders`
    fn load_colliders(&mut self, map: &tiled::Map, world: &mut World) {
        if let Some(ref group) = map
            .object_groups
            .iter()
            .filter(|&g| g.name == String::from("colliders"))
            .nth(0)
        {
            for obj in &group.objects {
                // Just do rectangles for now.
                if let tiled::ObjectShape::Rect { width, height, .. } = obj.shape {
                    let (x, y) = convert_tiled_xy(obj.x, obj.y);
                    let max = Point2::new(x + width, y);
                    let min = Point2::new(x, y - height);
                    let aabb = AABB::new(min, max);
                    println!("X {}, Y {}, MIN {:?}, MAX {:?}", x, y, min, max);

                    let mut debug_line = DebugLinesComponent::with_capacity(10);
                    debug_line.add_rectangle_2d(
                        min,
                        max,
                        DEBUG_LAYER,
                        Srgba::new(1.0, 0.0, 0.0, 1.0),
                    );

                    let entity = world
                        .create_entity()
                        .with(Obstacle { aabb })
                        .with(debug_line)
                        .build();
                    self.all_colliders.push(entity);
                }
            }
        }
    }

    /// Will load object from the tilemap.
    fn load_props(
        &mut self,
        map: &tiled::Map,
        world: &mut World,
        offsets: &Vec<u32>,
        spritesheets: &Vec<Handle<SpriteSheet>>,
    ) {
        if let Some(ref group) = map
            .object_groups
            .iter()
            .filter(|&g| g.name.to_lowercase() == String::from("props"))
            .nth(0)
        {
            for obj in &group.objects {
                let (x, y) = convert_tiled_xy(obj.x, obj.y);
                let tile = obj.gid;
                let tid = choose_tileset(tile, offsets);
                let offset = offsets[tid];
                let real_tile_id = tile - offset;

                let mut transform = Transform::default();
                transform.set_translation_xyz(x + 8.0, y + 8.0, PROPS_LAYER);

                let mut entity_builder = world
                    .create_entity()
                    .with(SpriteRender {
                        sprite_number: real_tile_id as usize,
                        sprite_sheet: spritesheets[tid].clone(),
                    })
                    .with(transform);

                match obj.obj_type.to_lowercase().as_str() {
                    "light" => {
                        if let Some(tiled::PropertyValue::StringValue(ref anim)) =
                            obj.properties.get(&"animation".to_string())
                        {
                            let light_animation = Animation::new(
                                anim.split(",")
                                    .map(|el| el.parse::<usize>().unwrap())
                                    .collect::<Vec<_>>(),
                                0.7,
                            );
                            let mut animation_controller = AnimationController::default();
                            animation_controller
                                .animations
                                .insert("idle".to_string(), light_animation);
                            animation_controller.current_animation = Some("idle".to_string());
                            entity_builder = entity_builder.with(animation_controller);
                        }
                    }
                    _ => (),
                }

                let entity = entity_builder.build();
                self.all_props.push(entity);
            }
        }
    }
}

fn convert_tiled_xy(x: f32, y: f32) -> (f32, f32) {
    (x - 1.0, ARENA_HEIGHT - (y - 1.0))
}

/// Will choose the correct tileset for the given tile id
/// offsets should be sorted from smallest to largest
fn choose_tileset(tile_id: u32, offsets: &Vec<u32>) -> usize {
    let mut tileset_idx = 0;

    // offset should be the largest offset inferior to tile_id.
    for (i, &offset) in offsets.iter().enumerate() {
        if tile_id < offset {
            break;
        }

        tileset_idx = i;
    }

    return tileset_idx;
}