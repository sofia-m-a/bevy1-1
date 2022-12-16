pub mod brushes;
pub mod level_graph;
pub mod physics;

use crate::{
    assets::{SpriteAssets, SHEET_W, TILE_SIZE, SHEET_H},
    camera::SofiaCamera,
    //map::brushes::*,
};
use bevy::{
    math::{IVec2, Rect, Vec3},
    prelude::*,
};
use bevy_ecs_tilemap::{tiles::{TileStorage, TileBundle, TilePos, TileTextureIndex, TileVisible, TileFlip}, prelude::{TilemapSize, TilemapId, TilemapGridSize, TilemapType, TilemapTexture, TilemapTileSize}, TilemapBundle};
use bevy_rapier2d::prelude::*;
use brushes::*;
use extent::Extent;
use rand_pcg::Pcg64;
use std::collections::HashSet;

use self::level_graph::{gen_graph, layout_graph};

// must be a power of two for Morton encoding to work
// otherwise we need to change CHUNK_SIZE^2 below to to_nearest_pow2(CHUNK_SIZE)^2
pub const CHUNK_SIZE: usize = 32;

pub type Place = IVec2;

pub fn intersect(r: Rect) -> impl Iterator<Item = Place> {
    let chunk_start_x = f32::floor(r.min.x / CHUNK_SIZE as f32) as i32;
    let chunk_end_x = f32::ceil(r.max.x / CHUNK_SIZE as f32) as i32;
    let chunk_start_y = f32::floor(r.max.y / CHUNK_SIZE as f32) as i32;
    let chunk_end_y = f32::ceil(r.min.y / CHUNK_SIZE as f32) as i32;

    itertools::iproduct!(chunk_start_x..=chunk_end_x, chunk_start_y..=chunk_end_y)
        .map(|(a, b)| Place::new(a, b))
}

#[derive(Clone, Copy, PartialEq, Component)]
pub struct ActiveTile {
    pub tile: Tile,
    pub flip: Flip,
    pub tint: Color,
}

impl Default for ActiveTile {
    fn default() -> Self {
        Self {
            tile: Tile::Air,
            flip: Flip::empty(),
            tint: Color::WHITE,
        }
    }
}

bitflags::bitflags! {
    pub struct Flip: u32 {
        const FLIP_H = 0b00000001;
        const FLIP_V = 0b00000010;
        const FLIP_D = 0b00000100;
    }
}

impl From<Tile> for ActiveTile {
    fn from(t: Tile) -> Self {
        Self {
            tile: t,
            flip: Flip::empty(),
            tint: Color::WHITE,
        }
    }
}

#[derive(Component)]
pub struct Chunk;

pub fn chunk_loader(
    mut commands: Commands,
    chunks: Query<(Entity, &Chunk, &Transform)>,
    mut res_gen: ResMut<Gen>,
    sa: Res<SpriteAssets>,
    views: Query<&SofiaCamera>,
) {
    if let Some(view) = views.iter().next() {
        let mut visible: HashSet<Place> = HashSet::from_iter(intersect(view.view));
        for c in chunks.iter() {
            let place = Place::new(
                (c.2.translation.x / (CHUNK_SIZE as f32 * TILE_SIZE as f32)) as i32,
                (c.2.translation.y / (CHUNK_SIZE as f32 * TILE_SIZE as f32)) as i32,
            );
            if visible.contains(&place) {
                visible.remove(&place);
            } else {
                // unload
                commands.entity(c.0).despawn_recursive();
            }
        }

        for &c in visible.iter() {

            // let mut children = Vec::new();

            // load
            let seed = res_gen.seed;
            let mut pcg = Pcg64::new(
                (c.x as u128).wrapping_add(seed),
                (c.y as u128).wrapping_add(seed),
            );
            //let tiles = heightmap_ground(c, &mut pcg, res_gen.terrain);
            let tiles = igloo(Extent::new(0, CHUNK_SIZE as i32), 0, &mut pcg);

            let mut storage = TileStorage::empty(TilemapSize { x: CHUNK_SIZE as u32, y: CHUNK_SIZE as u32 });
            let tilemap_entity = commands.spawn_empty().id();

            for (p, t) in tiles {
                if t == Tile::Air { continue }

                let tile_pos = TilePos { x: p.x as u32, y: p.y as u32 };
                let (ix, iy) = t.into();
                let tile_entity = commands.spawn(TileBundle {
                    position: tile_pos,
                    texture_index: TileTextureIndex((iy*SHEET_W + ix) as u32),
                    tilemap_id: TilemapId(tilemap_entity),
                    visible: TileVisible(true),
                    ..Default::default()
                }).id();

                storage.set(&tile_pos, tile_entity);
            }

            commands.entity(tilemap_entity).insert(TilemapBundle {
                grid_size: TilemapGridSize { x: TILE_SIZE as f32, y: TILE_SIZE as f32 },
                map_type: TilemapType::Square,
                size: storage.size,
                storage,
                texture: TilemapTexture::Single(sa.),
                tile_size: TilemapTileSize { x: TILE_SIZE as f32, y: TILE_SIZE as f32 },
                transform: Transform::from_translation(Vec3::new(
                            c.x as f32 * CHUNK_SIZE as f32,
                            c.y as f32 * CHUNK_SIZE as f32,
                            1.0,
                        )),
                ..Default::default()
            });

            // let map_entity = commands.spawn(()).id();
            // let mut map = bevy_ecs_tilemap::Map::new(0u16, map_entity);

            // let (mut layer_builder, layer_entity) =  bevy_ecs_tilemap::LayerBuilder::<bevy_ecs_tilemap::TileBundle>::new(
            //     &mut commands,
            //     bevy_ecs_tilemap::LayerSettings::new(
            //         bevy_ecs_tilemap::MapSize(1, 1),
            //         bevy_ecs_tilemap::ChunkSize(CHUNK_SIZE as u32, CHUNK_SIZE as u32),
            //         bevy_ecs_tilemap::TileSize(TILE_SIZE as f32, TILE_SIZE as f32),
            //         bevy_ecs_tilemap::TextureSize(TILE_SIZE as f32 * SHEET_W as f32, TILE_SIZE as f32 * SHEET_H as f32),
            //     ),
            //     0u16,
            //     0u16,
            // );
            // layer_builder.set_all( bevy_ecs_tilemap::TileBundle::default());

            // for &(p, t) in tiles.iter() {
            //     if t == Tile::Air {
            //         continue;
            //     }
            //     let (ti, tj) = <(u16, u16)>::from(t);
            //     let pos = c * CHUNK_SIZE as i32 + p;

            //     let id = commands
            //         .spawn()
            //         .insert_bundle(SpriteSheetBundle {
            //             sprite: TextureAtlasSprite {
            //                 index: (ti + SHEET_W * tj) as usize,
            //                 ..Default::default()
            //             },
            //             texture_atlas: sa.tile_texture.clone(),
            //             transform: Transform::from_translation(Vec3::new(
            //                 pos.x as f32 * TILE_SIZE as f32,
            //                 pos.y as f32 * TILE_SIZE as f32,
            //                 1 as f32,
            //             )),
            //             ..Default::default()
            //         })
            //         .insert_bundle(ColliderBundle {
            //             shape: ColliderShapeComponent(
            //                 physics::mesh_for(Tile::Terrain(Terrain::Cake, TerrainTile::Block))
            //                     .unwrap(),
            //             ),
            //             position: ColliderPositionComponent(ColliderPosition(
            //                 Isometry::translation(pos.x as f32, pos.y as f32),
            //             )),
            //             ..Default::default()
            //         })
            //         .id();
            //     children.push(id);
            // }

            // commands
            //     .spawn()
            //     .insert(Chunk)
            //     .insert(Transform::from_translation(Vec3::new(
            //         c.x as f32 * CHUNK_SIZE as f32,
            //         c.y as f32 * CHUNK_SIZE as f32,
            //         1.0,
            //     )))
            //     .insert(GlobalTransform::default())
            //     .push_children(&children);
        }
    }
}
