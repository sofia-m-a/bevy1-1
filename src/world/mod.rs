pub mod brushes;
pub mod feature;
pub mod tile;

use bevy::{
    math::{IVec2, Rect, Vec3},
    prelude::*,
    sprite::Anchor,
};
use bevy_ecs_tilemap::{
    prelude::{
        fill_tilemap, TilemapGridSize, TilemapId, TilemapSize, TilemapTexture, TilemapTileSize,
        TilemapType,
    },
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
    TilemapBundle,
};
use brushes::*;
use noise::NoiseFn;
use std::collections::HashSet;

use self::{feature::*, tile::*};
use crate::{
    assets::{SpriteAssets, PIXEL_MODEL_TRANSFORM, SHEET_W, TILE_SIZE},
    camera::{get_camera_rect, LetterboxProjection, SofiaCamera},
    helpers::*,
};

pub const CHUNK_SIZE: usize = 32;

pub type Place = IVec2;

pub fn intersect(r: Rect) -> impl Iterator<Item = Place> {
    let chunk_start_x = f32::floor(r.min.x / CHUNK_SIZE as f32) as i32;
    let chunk_end_x = f32::floor(r.max.x / CHUNK_SIZE as f32) as i32;
    let chunk_start_y = f32::floor(r.min.y / CHUNK_SIZE as f32) as i32;
    let chunk_end_y = f32::floor(r.max.y / CHUNK_SIZE as f32) as i32;

    itertools::iproduct!(chunk_start_x..=chunk_end_x, chunk_start_y..=chunk_end_y)
        .map(|(a, b)| Place::new(a, b))
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Chunk;

fn debug_load_chunk(
    commands: &mut Commands,
    sa: Res<SpriteAssets>,
    chunk_place: Place,
) {
    let chunk = commands
        .spawn(Chunk)
        .insert(SpatialBundle::from_transform(Transform::from_translation(
            Vec3::new(
                chunk_place.x as f32 * CHUNK_SIZE as f32,
                chunk_place.y as f32 * CHUNK_SIZE as f32,
                0.0,
            ),
        )))
        .id();
    let mut storage = TileStorage::empty(TilemapSize { x: 20, y: 20 });
    let tilemap_entity = commands.spawn_empty().id();
    fill_tilemap(
        TileTextureIndex(0),
        storage.size,
        TilemapId(tilemap_entity),
        commands,
        &mut storage,
    );
    commands.entity(chunk).add_child(tilemap_entity);
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
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)) * PIXEL_MODEL_TRANSFORM,
        ..Default::default()
    });
}

fn feature_colorize(f: Feature, res_gen: Res<Gen>) -> Color {
    let b = f.bounds();
    let c = res_gen.theme.get([
        b.x.lo_incl as f64 + 100.0 * b.x.hi_excl as f64,
        b.y.hi_excl as f64 + 100.0 * b.y.hi_excl as f64,
    ]);
    Color::Hsla {
        hue: (360.0 * c) as f32,
        saturation: 1.0,
        lightness: 0.7,
        alpha: 1.0,
    }
}

#[derive(Clone, Copy, Component)]
struct Outline;

fn feature_outline(
    commands: &mut Commands,
    f: Feature,
    chunk: Entity,
    sa: Res<SpriteAssets>,
    color: Color,
) {
    let b = f.bounds();
    let t = 1.0 / 20.0;

    let o = commands
        .spawn(Outline)
        .insert(SpatialBundle::from_transform(Transform::from_xyz(
            0.0, 0.0, 10.0,
        )))
        .id();
    commands.entity(chunk).add_child(o);

    let bottom = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(b.x.size() as f32, t)),
                color,
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            texture: sa.blank_texture.clone(),
            transform: Transform::from_xyz(b.x.lo_incl as f32, b.y.lo_incl as f32, 0.0),
            ..Default::default()
        })
        .id();
    let left = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(t, b.y.size() as f32)),
                color,
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            texture: sa.blank_texture.clone(),
            transform: Transform::from_xyz(b.x.lo_incl as f32, b.y.lo_incl as f32, 0.0),
            ..Default::default()
        })
        .id();
    let right = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(t, b.y.size() as f32)),
                color,
                anchor: Anchor::TopRight,
                ..Default::default()
            },
            texture: sa.blank_texture.clone(),
            transform: Transform::from_xyz(b.x.hi_excl as f32, b.y.hi_excl as f32, 0.0),
            ..Default::default()
        })
        .id();
    let top = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(b.x.size() as f32, t)),
                color,
                anchor: Anchor::TopRight,
                ..Default::default()
            },
            texture: sa.blank_texture.clone(),
            transform: Transform::from_xyz(b.x.hi_excl as f32, b.y.hi_excl as f32, 0.0),
            ..Default::default()
        })
        .id();

    commands
        .entity(o)
        .add_child(left)
        .add_child(right)
        .add_child(top)
        .add_child(bottom);
}

fn feature_text(
    commands: &mut Commands,
    f: Feature,
    chunk: Entity,
    sa: Res<SpriteAssets>,
    string: &str,
    color: Color,
) {
    let b = f.bounds();

    let style = TextStyle {
        color,
        ..sa.text_style.clone()
    };

    let text = Text::from_section(string, style);

    let text = commands
        .spawn(Text2dBundle {
            text,
            transform: Transform::from_translation(b.center().extend(10.0))
                * Transform::from_scale(Vec3::new(1.0 / 60.0, 1.0 / 60.0, 1.0)),
            ..Default::default()
        })
        .id();
    commands.entity(chunk).add_child(text);
}

fn load_chunk(
    commands: &mut Commands,
    res_gen: Res<Gen>,
    sa: Res<SpriteAssets>,
    chunk_place: Place,
) {
    let bounds = Box2::from_box1s(
        Box1::new(
            chunk_place.x * CHUNK_SIZE as i32,
            (chunk_place.x + 1) * CHUNK_SIZE as i32,
        ),
        Box1::new(
            chunk_place.y * CHUNK_SIZE as i32,
            (chunk_place.y + 1) * CHUNK_SIZE as i32,
        ),
    );

    let place_vec = Vec2::new(
        chunk_place.x as f32 * CHUNK_SIZE as f32,
        chunk_place.y as f32 * CHUNK_SIZE as f32,
    );

    let schema = generate_level(&res_gen);
    let v = render_level(&schema, &res_gen, bounds);
    
    let chunk = commands.spawn(Chunk).insert(SpatialBundle::default()).id();

    let chunk_text = commands
        .spawn(Text2dBundle {
            text: Text::from_section(format!("Chunk {}", chunk_place), sa.text_style.clone()),
            transform: Transform::from_translation(
                bounds.center().extend(10.0) + Vec3::new(0.0, -10.0, 0.0),
            ) * Transform::from_scale(Vec3::new(1.0 / 60.0, 1.0 / 60.0, 1.0)),
            ..Default::default()
        })
        .id();
    commands.entity(chunk).add_child(chunk_text);

    for f in schema.intersecting(bounds) {
        let c = feature_colorize(f, Res::clone(&res_gen));
        match f {
            Feature::Zone(z, _) => {
                feature_text(
                    commands,
                    f,
                    chunk,
                    Res::clone(&sa),
                    &format!("{:?}", z),
                    Color::WHITE,
                );
                //feature_outline(commands, f, chunk, chunk_place, Res::clone(&sa), Color::WHITE);
            }
            Feature::HillBlock { .. } | Feature::GroundBlock(..) => {
                feature_outline(commands, f, chunk, Res::clone(&sa), c);
            }
            _ => (),
        }
    }

    let mut storage_and_entity = Vec::with_capacity(v.dim().2);
    for _ in 0..v.dim().2 {
        let storage = TileStorage::empty(TilemapSize {
            x: v.dim().0 as u32,
            y: v.dim().1 as u32,
        });
        let tilemap_entity = commands.spawn_empty().id();
        storage_and_entity.push((storage, tilemap_entity));
        commands.entity(chunk).add_child(tilemap_entity);
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
            transform: Transform::from_translation(
                (place_vec + Vec2::new(0.5, 0.5)).extend(k as f32),
            ) * PIXEL_MODEL_TRANSFORM,
            ..Default::default()
        });
    }
}

pub fn chunk_loader(
    mut commands: Commands,
    mut chunks: Query<(Entity, &Transform), With<Chunk>>,
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
            load_chunk(
                &mut commands,
                Res::clone(&res_gen),
                Res::clone(&sa),
                c,
            );
        }
    }
}
