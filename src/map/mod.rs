pub mod brushes;
pub mod level_graph;
pub mod physics;

use crate::{
    assets::{SpriteAssets, PIXEL_MODEL_TRANSFORM, SHEET_H, SHEET_W, TILE_SIZE},
    camera::{get_camera_rect, LetterboxProjection, SofiaCamera},
    //map::brushes::*,
};
use bevy::{
    math::{IVec2, Rect, Vec3},
    prelude::*,
};
use bevy_ecs_tilemap::{
    prelude::{
        fill_tilemap, TilemapGridSize, TilemapId, TilemapSize, TilemapTexture, TilemapTileSize,
        TilemapType,
    },
    tiles::{TileBundle, TileFlip, TilePos, TileStorage, TileTextureIndex, TileVisible},
    TilemapBundle,
};
use bevy_rapier2d::prelude::*;
use brushes::*;
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

#[derive(Component, Clone, Copy, Debug)]
pub struct Chunk;

#[derive(Copy, Clone, Component, PartialEq, Eq, Debug, Hash)]
pub struct LevelEntity;

#[derive(Clone, Copy, Resource)]
pub struct LevelResource(pub Entity);

pub fn add_level_resource(mut commands: Commands) {
    let entity = commands
        .spawn(LevelEntity)
        .insert(SpatialBundle::default())
        .id();
    commands.insert_resource(LevelResource(entity));
}

fn load_chunk(
    commands: &mut Commands,
    level: Res<LevelResource>,
    res_gen: Res<Gen>,
    sa: Res<SpriteAssets>,
    chunk: Place,
) {
    let schema = generate_level(&res_gen);
    let arr = render_level(&schema, &res_gen, schema.bounds());
    let v = arr.slice(ndarray::s![1isize..-1, 1isize..-1, ..]);
    let chunk = commands
        .spawn(Chunk)
        .insert(SpatialBundle::from_transform(Transform::from_translation(Vec3::new(
            chunk.x as f32 * CHUNK_SIZE as f32,
            chunk.y as f32 * CHUNK_SIZE as f32,
            0.0,
        ))))
        .id();
    let tilemap = commands.spawn(SpatialBundle::default()).id();
    //.insert(RigidBody::Fixed)
    commands.entity(chunk).add_child(tilemap);
    commands.entity(level.0).add_child(tilemap);

    let mut storage_and_entity = Vec::with_capacity(v.dim().2);
    for _ in 0..v.dim().2 {
        let storage = TileStorage::empty(TilemapSize {
            x: v.dim().0 as u32,
            y: v.dim().1 as u32,
        });
        let tilemap_entity = commands.spawn_empty().id();
        storage_and_entity.push((storage, tilemap_entity));
        commands.entity(tilemap).add_child(tilemap_entity);
    }

    for ((i, j, k), &t) in v.indexed_iter() {
        if t == Tile::Air {
            continue;
        }

        let tile_pos = TilePos {
            x: i as u32,
            y: j as u32,
        };
        let (ix, iy) = t.into();
        let tile_entity = commands
            .spawn(TileBundle {
                position: tile_pos,
                texture_index: TileTextureIndex((iy * SHEET_W + ix) as u32),
                tilemap_id: TilemapId(storage_and_entity[k].1),
                ..Default::default()
            })
            .id();

        storage_and_entity[k].0.set(&tile_pos, tile_entity);
        commands
            .entity(storage_and_entity[k].1)
            .add_child(tile_entity);
    }

    for (k, (storage, tilemap_entity)) in storage_and_entity.into_iter().enumerate() {
        commands.entity(tilemap_entity).insert(TilemapBundle {
            grid_size: TilemapGridSize {
                x: TILE_SIZE as f32,
                y: TILE_SIZE as f32,
            },
            map_type: TilemapType::Square,
            size: storage.size,
            storage,
            texture: TilemapTexture::Single(sa.tile_texture.clone()),
            tile_size: TilemapTileSize {
                x: TILE_SIZE as f32,
                y: TILE_SIZE as f32,
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, k as f32)) * PIXEL_MODEL_TRANSFORM,
            ..Default::default()
        });
    }

    // for &f in schema.features.iter() {
    //     // if let Some(collider) = collider_for(f) {
    //     //     //let collider = commands.spawn(collider).id();
    //     //     //commands.entity(level).add_child(collider);
    //     // }
    // }
}

pub fn chunk_loader(
    mut commands: Commands,
    mut chunks: Query<(Entity, &Transform), With<Chunk>>,
    level: Res<LevelResource>,
    res_gen: Res<Gen>,
    sa: Res<SpriteAssets>,
    views: Query<(&Transform, &LetterboxProjection), With<SofiaCamera>>,
) {
    for view in views.iter() {
        let mut visible: HashSet<Place> =
            HashSet::from_iter(intersect(get_camera_rect(view.0, view.1)));

        for c in chunks.iter_mut() {
            let place = Place::new(
                (c.1.translation.x / (CHUNK_SIZE as f32)) as i32,
                (c.1.translation.y / (CHUNK_SIZE as f32)) as i32,
            );
            if visible.contains(&place) {
                visible.remove(&place);
            } else {
                // unload
                commands.entity(c.0).despawn_recursive();
            }
        }

        for &c in visible.iter() {
            load_chunk(&mut commands, Res::clone(&level), Res::clone(&res_gen), Res::clone(&sa), c);
        }
    }
}

// pub fn chunk_loader(
//     mut commands: Commands,
//     mut chunks: Query<(Entity, &Chunk, &Transform, &mut TileStorage)>,
//     res_gen: ResMut<Gen>,
//     sa: Res<SpriteAssets>,
//     views: Query<&SofiaCamera>,
// ) {

//         for &c in visible.iter() {
//             // load
//             let seed = res_gen.seed;
//             let mut pcg = Pcg64::new(
//                 (c.x as u128).wrapping_add(seed),
//                 (c.y as u128).wrapping_add(seed),
//             );
//             //let tiles = slopey_ground(c, &mut pcg, res_gen.terrain);
//             let tiles: Vec<(Place, Tile)> = Vec::new(); // igloo(Extent::new(0, CHUNK_SIZE as i32), 0, &mut pcg);

//             let mut storage = TileStorage::empty(TilemapSize {
//                 x: CHUNK_SIZE as u32,
//                 y: CHUNK_SIZE as u32,
//             });
//             let tilemap_entity = commands.spawn_empty().id();

//             for (p, t) in tiles {
//                 if t == Tile::Air {
//                     continue;
//                 }

//                 let tile_pos = TilePos {
//                     x: p.x as u32,
//                     y: p.y as u32,
//                 };
//                 let (ix, iy) = t.into();
//                 let tile_entity = commands
//                     .spawn(TileBundle {
//                         position: tile_pos,
//                         texture_index: TileTextureIndex((iy * SHEET_W + ix) as u32),
//                         tilemap_id: TilemapId(tilemap_entity),
//                         visible: TileVisible(true),
//                         ..Default::default()
//                     })
//                     .id();

//                 storage.set(&tile_pos, tile_entity);
//             }

//             commands.entity(tilemap_entity).insert(TilemapBundle {
//                 grid_size: TilemapGridSize {
//                     x: TILE_SIZE as f32,
//                     y: TILE_SIZE as f32,
//                 },
//                 map_type: TilemapType::Square,
//                 size: storage.size,
//                 storage,
//                 texture: TilemapTexture::Single(sa.tile_texture.clone()),
//                 tile_size: TilemapTileSize {
//                     x: TILE_SIZE as f32,
//                     y: TILE_SIZE as f32,
//                 },
//                 transform: Transform::from_translation(Vec3::new(
//                     c.x as f32 * CHUNK_SIZE as f32 * TILE_SIZE as f32,
//                     c.y as f32 * CHUNK_SIZE as f32 * TILE_SIZE as f32,
//                     1.0,
//                 )),

//                 visibility: Visibility::VISIBLE,
//                 ..Default::default()
//             });
//         }
//     }
// }
