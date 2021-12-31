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

pub type Chunk<T> = TooDee<T>;

pub struct SparseGrid<T> {
    pub grid: HashMap<Coordinate, Chunk<T>>,
    pub def: T,
}

impl<T> SparseGrid<T> {
    pub fn new(default: T) -> Self {
        Self {
            grid: HashMap::new(),
            def: default,
        }
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
            .map(|chunk| chunk.index((into_chunk(i.x) as usize, into_chunk(i.y) as usize)))
            .unwrap_or(&self.def)
    }
}

impl<T> IndexMut<Coordinate> for SparseGrid<T> {
    fn index_mut(&mut self, i: Coordinate) -> &mut T {
        let shifted = i / (CHUNK_SIZE as i32);
        self.grid
            .get_mut(&shifted)
            .map(|chunk| chunk.index_mut((into_chunk(i.x) as usize, into_chunk(i.y) as usize)))
            .unwrap_or(&mut self.def)
    }
}
fn layer_settings() -> LayerSettings {
    LayerSettings::new(
        UVec2::new(1, 1),
        UVec2::new(CHUNK_SIZE as u32, CHUNK_SIZE as u32),
        Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
        Vec2::new(
            (TILE_SIZE * SHEET_W as u32) as f32,
            (TILE_SIZE * SHEET_H as u32) as f32,
        ),
    )
}

pub fn generate_chunk(
    commands: &mut Commands,
    map_query: &mut MapQuery,
    texture: Handle<ColorMaterial>,
    map_id: u16,
) {
    let map_entity = commands.spawn().id();
    let map = Map::new(map_id, map_entity);

    let (mut layer_builder_bg, _) = LayerBuilder::new(commands, layer_settings(), map_id, 0u16);
    let (mut layer_builder_fg, _) = LayerBuilder::new(commands, layer_settings(), map_id, 1u16);

    generate_island(&mut layer_builder_bg, &mut layer_builder_fg).unwrap();

    let layer_entity_bg = map_query.build_layer(commands, layer_builder_bg, texture.clone());
    let layer_entity_fg = map_query.build_layer(commands, layer_builder_fg, texture.clone());

    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-128.0, -128.0, 0.0))
        .insert(GlobalTransform::default());
}

pub fn unload_chunk(commands: &mut Commands, query: &mut MapQuery, map_id: u16) {
    query.depsawn_layer(commands, map_id, 0u16);
    query.depsawn_map(commands, map_id);
    query.despawn_layer_tiles(commands, map_id, 0u16);
    query.despawn_tile(commands, UVec2::ZERO, 0u16, 0u16);
    query.notify_chunk_for_tile(UVec2::ZERO, 0u16, 0u16);
    println!("theere");
}
