use lindel::{morton_decode, morton_encode};
use std::{
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
};

use bevy::{
    math::{IVec2, Rect, Vec2, Vec3},
    prelude::*,
};

use crate::{
    assets::{SpriteAssets, TILE_SIZE},
    camera::WorldView,
};

use super::{brushes::TileType, gen::generate_island};

pub const CHUNK_SIZE: usize = 32;

pub type Coordinate = IVec2;

#[derive(Component)]
pub struct Map {
    pub chunks: HashMap<Coordinate, Chunk>,
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

pub struct Chunk {
    pub grid: [Tile; CHUNK_SIZE * CHUNK_SIZE],
}

impl Index<(u32, u32)> for Chunk {
    type Output = Tile;
    fn index(&self, s: (u32, u32)) -> &Tile {
        &self.grid[morton_encode([s.0, s.1]) as usize]
    }
}

impl IndexMut<(u32, u32)> for Chunk {
    fn index_mut(&mut self, s: (u32, u32)) -> &mut Tile {
        &mut self.grid[morton_encode([s.0, s.1]) as usize]
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Tile {
    pub layers: [TileDesc; 3],
}

impl Default for Tile {
    fn default() -> Self {
        let s = TileDesc {
            tile: TileType::Air,
            flip: Flip::empty(),
            tint: Color::WHITE,
            entity: None,
        };
        Self { layers: [s, s, s] }
    }
}

#[derive(Clone, Copy, PartialEq, Component)]
pub struct TileDesc {
    pub tile: TileType,
    pub flip: Flip,
    pub tint: Color,
    pub entity: Option<Entity>,
}

bitflags::bitflags! {
    pub struct Flip: u32 {
        const FLIP_H = 0b00000001;
        const FLIP_V = 0b00000010;
        const FLIP_D = 0b00000100;
    }
}

impl From<TileType> for TileDesc {
    fn from(t: TileType) -> Self {
        Self {
            tile: t,
            flip: Flip::empty(),
            tint: Color::WHITE,
            entity: None,
        }
    }
}

fn render(c: &mut Chunk, commands: &mut Commands, sa: &Res<SpriteAssets>, coord: IVec2) {
    for (i, tile) in c.grid.iter_mut().enumerate() {
        for (index, layer) in tile.layers.iter_mut().enumerate() {
            if layer.tile == TileType::Air {
                continue;
            }
            let entity = layer
                .entity
                .unwrap_or_else(|| commands.spawn().insert(*layer).id());

            let [a, b]: [u32; 2] = morton_decode(i as u64);

            commands.entity(entity).insert_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: layer.tint,
                    index: u16::from(layer.tile) as usize,
                    flip_x: layer.flip.contains(Flip::FLIP_H),
                    flip_y: layer.flip.contains(Flip::FLIP_V),
                },
                texture_atlas: sa.tile_texture.clone(),
                transform: Transform::from_translation(Vec3::new(
                    (a + coord.x as u32) as f32 * (TILE_SIZE as f32),
                    (b + coord.y as u32) as f32 * (TILE_SIZE as f32),
                    index as f32,
                )),
                ..Default::default()
            });
        }
    }
}

fn unload(c: &mut Chunk, commands: &mut Commands) {
    for tile in c.grid.iter_mut() {
        for l in tile.layers.iter_mut() {
            if let Some(t) = l.entity {
                commands.entity(t).remove_bundle::<SpriteSheetBundle>();
            }
        }
    }
}

pub fn chunk_load_unload(
    mut commands: Commands,
    mut maps: Query<&mut Map>,
    sa: Res<SpriteAssets>,
    views: Query<&WorldView>,
) {
    if let Some(view) = views.iter().next() {
        for mut map in maps.iter_mut() {
            let (must_make, must_delete) = map.get_changes(view.0);
            println!("{:?} {:?}", must_make, must_delete);
            for coord in must_make.iter() {
                let mut chunk = Chunk {
                    grid: [Tile::default(); CHUNK_SIZE * CHUNK_SIZE], //TODO: stack overflow for large chunk sizes
                };
                generate_island(&mut chunk);
                render(&mut chunk, &mut commands, &sa, *coord * (CHUNK_SIZE as i32));
                map.chunks.insert(*coord, chunk);
            }

            for coord in must_delete.iter() {
                if let Some(mut chunk) = map.chunks.get_mut(coord) {
                    unload(&mut chunk, &mut commands)
                }
            }
        }
    }
}

// fn render(c: &Chunk, index: u8) {
//     let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
//     let mut indices: Vec<u16> = Vec::new();
//     let mut vertexes: Vec<[f32; 3]> = Vec::new();
//     let mut colors: Vec<[f32; 4]> = Vec::new();
//     let mut texture: Vec<[u32; 2]> = Vec::new();

//     for i in 0..=CHUNK_SIZE {
//         for j in 0..=CHUNK_SIZE {
//             let tile = c.grid[(i, j)];
//             let tile = if index == 0 {
//                 tile.back
//             } else if index == 1 {
//                 tile.main
//             } else if index == 2 {
//                 tile.front
//             } else {
//                 panic!("sort out away to make this type safe")
//             };
//             let position = Vec3::new(i as f32, j as f32, index as f32);

//             indices.push(((i + CHUNK_SIZE * j) * 4 + 0) as u16);
//             indices.push(((i + CHUNK_SIZE * j) * 4 + 2) as u16);
//             indices.push(((i + CHUNK_SIZE * j) * 4 + 1) as u16);
//             indices.push(((i + CHUNK_SIZE * j) * 4 + 1) as u16);
//             indices.push(((i + CHUNK_SIZE * j) * 4 + 2) as u16);
//             indices.push(((i + CHUNK_SIZE * j) * 4 + 3) as u16);

//             vertexes.push((position).into());
//             vertexes.push((position + Vec3::X).into());
//             vertexes.push((position + Vec3::Y).into());
//             vertexes.push((position + Vec3::X + Vec3::Y).into());

//             colors.push(tile.tint.into());

//             texture.push([u16::from(tile.tile) as u32, tile.flip.bits()]);
//         }
//     }

//     mesh.set_attribute(
//         "Vertex_Position",
//         VertexAttributeValues::Float32x3(vertexes),
//     );
//     mesh.set_indices(Some(Indices::U16(indices)));
// }
