use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};
use toodee::TooDee;

use crate::{
    assets::{SHEET_H, SHEET_W, TILE_SIZE},
    gen::generate_island,
};

pub const CHUNK_SIZE: usize = 128;

pub type Coordinate = IVec2;

#[derive(Clone)]
pub struct Chunk<T> {
    grid: TooDee<T>,
    id: u16,
}

pub struct SparseGrid<T> {
    pub grid: HashMap<Coordinate, Chunk<T>>,
    pub def: Chunk<T>,
    pub def_cell: T,
}

fn find_visible_chunks(viewport: Rect<f32>) -> impl Iterator<Item = Coordinate> {
    let chunk_start_x = f32::floor(viewport.left / CHUNK_SIZE as f32) as i32;
    let chunk_end_x = f32::ceil(viewport.right / CHUNK_SIZE as f32) as i32;
    let chunk_start_y = f32::floor(viewport.bottom / CHUNK_SIZE as f32) as i32;
    let chunk_end_y = f32::ceil(viewport.top / CHUNK_SIZE as f32) as i32;

    itertools::iproduct!(chunk_start_x..=chunk_end_x, chunk_start_y..=chunk_end_y)
        .map(|(a, b)| Coordinate::new(a, b))
}

impl<T> SparseGrid<T> {
    pub fn new(default: Chunk<T>, cell: T) -> Self {
        Self {
            grid: HashMap::new(),
            def: default,
            def_cell: cell,
        }
    }
}

impl<T> SparseGrid<T>
where
    Chunk<T>: Clone,
{
    fn update_sparse_cache(&mut self, viewport: Rect<f32>) -> (Vec<Coordinate>, Vec<u16>) {
        let mut h = HashMap::new();
        let mut v = Vec::new();
        let mut d = Vec::new();
        for c in find_visible_chunks(viewport) {
            if let Some(chunk) = self.grid.remove(&c) {
                h.insert(c, chunk);
            } else {
                h.insert(c, self.def.clone());
                v.push(c);
            }
        }
        for (_, chunk) in self.grid.drain() {
            d.push(chunk.id);
        }

        self.grid = h;
        (v, d)
    }
}

impl<T: Copy> SparseGrid<T> {
    pub fn moore(&self, index: Coordinate) -> TooDee<T> {
        let i = IVec2::X;
        let j = IVec2::Y;

        TooDee::from_vec(
            3,
            3,
            vec![
                self[index - j - i],
                self[index - j],
                self[index - j + i],
                self[index - i],
                self[index],
                self[index + i],
                self[index + j - i],
                self[index + j],
                self[index + j + i],
            ],
        )
    }
}

fn into_chunk(i: i32) -> i32 {
    i % CHUNK_SIZE as i32
}

impl<T> Index<Coordinate> for SparseGrid<T> {
    type Output = T;
    fn index(&self, i: Coordinate) -> &T {
        let shifted = i / (CHUNK_SIZE as i32);
        self.grid
            .get(&shifted)
            .map(|chunk| {
                chunk
                    .grid
                    .index((into_chunk(i.x) as usize, into_chunk(i.y) as usize))
            })
            .unwrap_or(&self.def_cell)
    }
}

impl<T> IndexMut<Coordinate> for SparseGrid<T> {
    fn index_mut(&mut self, i: Coordinate) -> &mut T {
        let shifted = i / (CHUNK_SIZE as i32);
        self.grid
            .get_mut(&shifted)
            .map(|chunk| {
                chunk
                    .grid
                    .index_mut((into_chunk(i.x) as usize, into_chunk(i.y) as usize))
            })
            .unwrap_or(&mut self.def_cell)
    }
}

fn layer_settings(layer_id: u16) -> LayerSettings {
    let mut l = LayerSettings::new(
        MapSize(1, 1),
        ChunkSize(CHUNK_SIZE as u32, CHUNK_SIZE as u32),
        TileSize(TILE_SIZE as f32, TILE_SIZE as f32),
        TextureSize(
            (TILE_SIZE * SHEET_W as u32) as f32,
            (TILE_SIZE * SHEET_H as u32) as f32,
        ),
    );
    l.set_layer_id(layer_id);
    l
}

pub fn generate_chunk(
    commands: &mut Commands,
    map_query: &mut MapQuery,
    texture: Handle<Image>,
    map_id: u16,
) {
    let map_entity = commands.spawn().id();
    let mut map = Map::new(map_id, map_entity);

    let (mut layer_builder_bg, layer_entity_bg) =
        LayerBuilder::new(commands, layer_settings(0u16), map_id, 0u16);
    let (mut layer_builder_fg, layer_entity_fg) =
        LayerBuilder::new(commands, layer_settings(1u16), map_id, 1u16);

    generate_island(&mut layer_builder_bg, &mut layer_builder_fg).unwrap();

    map_query.build_layer(commands, layer_builder_bg, texture.clone());
    map_query.build_layer(commands, layer_builder_fg, texture.clone());

    map.add_layer(commands, 0u16, layer_entity_bg);
    map.add_layer(commands, 1u16, layer_entity_fg);

    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-128.0, -128.0, 0.0))
        .insert(GlobalTransform::default());
}

pub fn unload_chunk(commands: &mut Commands, query: &mut MapQuery, map_id: u16) {
    query.despawn(commands, map_id);
}

#[derive(Clone, Copy, Component)]
pub struct HighestMapID(u16);

pub fn regenerate_chunks(
    mut commands: Commands,
    mut query: MapQuery,
    mut h: ResMut<HighestMapID>,
    mut s: SparseGrid<Tile>,
    cameras: Query<&OrthographicProjection, With<Camera>>,
) {
    let map_id = *h;
    h.0 += 1;

    if let Some(cam) = cameras.iter().next() {
        let viewport = Rect {
            left: cam.left,
            right: cam.right,
            top: cam.top,
            bottom: cam.bottom,
        };
        let (make, destroy) = s.update_sparse_cache(viewport);
        for i in destroy {
            query.despawn(&mut commands, i);
        }
    }
}
