use bevy::math::IVec2;
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};
use toodee::TooDee;

pub const CHUNK_SIZE: usize = 32;

pub type Ordinate = i32;
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
