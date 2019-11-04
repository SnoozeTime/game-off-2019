use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::transform::ParentHierarchy,
    ecs::{
        error::WrongGeneration,
        prelude::{Entity, World, WorldExt},
    },
    renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
};

use std::iter;

/// delete the specified root entity and all of its descendents as specified
/// by the Parent component and maintained by the ParentHierarchy resource
// from https://github.com/amethyst/evoli src/utils/hierarchy_util.rs
pub fn delete_hierarchy(root: Entity, world: &mut World) -> Result<(), WrongGeneration> {
    let entities = {
        iter::once(root)
            .chain(
                world
                    .read_resource::<ParentHierarchy>()
                    .all_children_iter(root),
            )
            .collect::<Vec<Entity>>()
    };
    world.delete_entities(&entities)
}

/// Load the texture from the name
pub fn load_spritesheet(texture_name: &str, world: &mut World) -> Handle<SpriteSheet> {
    let texture_path = format!("texture/{}.png", texture_name);
    let ron_path = format!("texture/{}.ron", texture_name);

    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(texture_path, ImageFormat::default(), (), &texture_storage)
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_path,
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}
