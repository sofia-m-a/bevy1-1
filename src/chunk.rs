use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use toodee::TooDeeOps;

use crate::assets::Tile;
use crate::assets::TILE_SIZE;
use crate::brushes::*;
use crate::grid::*;
pub struct ChunkMarker(pub Entity);

#[derive(Clone, Copy)]
pub struct Cell {
    tile: Tile,
}

pub fn load_chunk(
    chunk: Chunk<Cell>,
    commands: &mut Commands,
    graphics: &Handle<TextureAtlas>,
    offset: Vec2,
) {
    let mut chunk_head = commands.spawn();
    let chunk_ref = chunk_head.id();
    chunk_head.insert(chunk_ref);

    for (i, j) in itertools::iproduct!(0..chunk.num_cols(), 0..chunk.num_rows()) {
        let tile = chunk[(i, j)];
        let offset = offset + Vec2::new(i as f32, j as f32).into();

        if tile.tile == Tile::Air {
            continue;
        }

        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: tile.tile as u32,
                    ..Default::default()
                },
                texture_atlas: graphics.clone(),
                transform: Transform::from_translation(Vec3::new(
                    offset.x * TILE_SIZE,
                    offset.y * TILE_SIZE,
                    0.0,
                )),
                ..Default::default()
            })
            .insert(ChunkMarker(chunk_ref));

        commands.spawn_bundle(ColliderBundle {
            collider_type: ColliderType::Solid,
            shape: ColliderShape::cuboid(1.0 / 2.0, 1.0 / 2.0),
            mass_properties: ColliderMassProps::Density(0.0),
            ..Default::default()
        });
    }
}

