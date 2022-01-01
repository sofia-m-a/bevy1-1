use std::collections::{HashMap, HashSet};

use bevy::{
    math::{IVec2, Rect},
    prelude::Color,
};
use toodee::TooDee;

pub const CHUNK_SIZE: usize = 128;

pub type Coordinate = IVec2;

struct Map {
    chunks: HashMap<Coordinate, Chunk>,
}

impl Map {
    pub fn new() -> Self {
        Map {
            chunks: HashMap::new(),
        }
    }

    pub fn intersect(r: Rect<f32>) -> Vec<Coordinate> {
        let chunk_start_x = f32::floor(r.left / CHUNK_SIZE as f32) as i32;
        let chunk_end_x = f32::ceil(r.right / CHUNK_SIZE as f32) as i32;
        let chunk_start_y = f32::floor(r.bottom / CHUNK_SIZE as f32) as i32;
        let chunk_end_y = f32::ceil(r.top / CHUNK_SIZE as f32) as i32;

        itertools::iproduct!(chunk_start_x..=chunk_end_x, chunk_start_y..=chunk_end_y)
            .map(|(a, b)| Coordinate::new(a, b))
            .collect()
    }

    pub fn get_changes(&self, r: Rect<f32>) -> (HashSet<IVec2>, HashSet<IVec2>) {
        let c: HashSet<IVec2> = HashSet::from_iter(Self::intersect(r).iter().copied());
        let d: HashSet<IVec2> = HashSet::from_iter(self.chunks.keys().copied());

        let must_make = HashSet::from_iter(c.difference(&d).copied());
        let must_delete = HashSet::from_iter(d.difference(&c).copied());
        (must_make, must_delete)
    }
}

struct Chunk {
    grid: TooDee<Tile>,
}

struct Tile {
    back: TileDesc,
    main: TileDesc,
    front: TileDesc,
}

struct TileDesc {
    index: u16,
    flip_h: bool,
    flip_v: bool,
    flip_d: bool,
    tint: Color,
}
