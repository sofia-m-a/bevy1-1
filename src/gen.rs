use std::ops::Range;

use bevy::prelude::*;
use bevy_ecs_tilemap::{LayerBuilder, MapTileError, TileBundle};
use noise::{NoiseFn, Perlin, Seedable};
use rand::prelude::*;
use rand_pcg::Pcg64;

use crate::{
    assets::SpriteAssets,
    brushes::{GroundSet, GroundTileType, IglooPiece, Slope, Tile, LMR, LR},
    grid::CHUNK_SIZE,
};

fn place_tile(
    b: &mut LayerBuilder<TileBundle>,
    i: u32,
    j: u32,
    tile: Tile,
) -> Result<(), MapTileError> {
    b.set_tile(
        UVec2::new(i, j),
        TileBundle {
            tile: bevy_ecs_tilemap::Tile {
                texture_index: u16::from(tile),
                ..Default::default()
            },
            ..Default::default()
        },
    )?;
    Ok(())
}

fn place_igloo(
    b: &mut LayerBuilder<TileBundle>,
    rng: &mut Pcg64,
    width: Range<u32>,
    height: Range<u32>,
) -> Result<(), MapTileError> {
    for i in width.clone() {
        for j in height.clone() {
            let left = i == width.clone().start;
            let right = i == width.clone().end - 1;
            let top = j == height.clone().end - 1;
            let piece = if top {
                IglooPiece::Top(if left {
                    LMR::Left
                } else if right {
                    LMR::Right
                } else {
                    LMR::Mid
                })
            } else {
                *[IglooPiece::Interior, IglooPiece::InteriorAlt]
                    .choose(rng)
                    .unwrap()
            };
            place_tile(b, i, j, Tile::Igloo(piece))?;
        }
    }

    if height.len() >= 2 {
        let door_location = rng.gen_range(width);
        place_tile(
            b,
            door_location,
            height.start,
            Tile::Igloo(IglooPiece::Door),
        )?;
    }

    Ok(())
}

pub fn generate_island(
    b: &mut LayerBuilder<TileBundle>,
    fg: &mut LayerBuilder<TileBundle>,
) -> Result<(), MapTileError> {
    let mut tr = thread_rng();
    let mut rng = Pcg64::from_rng(&mut tr).unwrap();
    let noise = noise::OpenSimplex::new().set_seed(rng.next_u32());

    let set = GroundSet::Tundra;

    let mut height_map = [0; CHUNK_SIZE];

    for i in 0..CHUNK_SIZE {
        let height = (12.0 * noise.get([(i as f64) * 0.04, i as f64 * 0.04]).abs()) as u32;
        height_map[i] = height;
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Type {
        Flat,
        Slope(Slope),
    }

    let mut type_map = [Type::Flat; CHUNK_SIZE];
    for i in 0..CHUNK_SIZE {
        let left = height_map[i.saturating_sub(1)];
        let right = height_map[(i + 1).min(CHUNK_SIZE - 1)];
        let current = height_map[i];

        let slope_up = right == left + 1 && current == right;
        let slope_down = left == right + 1 && current == left;

        type_map[i] = if slope_up {
            Type::Slope(Slope::UpRight)
        } else if slope_down {
            Type::Slope(Slope::DownLeft)
        } else {
            Type::Flat
        };
    }

    for i in 0..CHUNK_SIZE {
        let current = height_map[i];
        for j in 0..=current {
            let top = j == current;
            let top1 = current != 0 && j == (current - 1);

            let tile = match type_map[i] {
                Type::Slope(s) if top => GroundTileType::Slope(s),
                Type::Slope(s) if top1 => GroundTileType::SlopeInt(s),
                Type::Flat if top => GroundTileType::Ground(LMR::Mid),
                _ => GroundTileType::Interior,
            };

            place_tile(b, i as u32, j, Tile::Ground(tile, set))?;
        }
    }

    for i in 0..CHUNK_SIZE {
        let left = height_map[i.saturating_sub(1)];
        let right = height_map[(i + 1).min(CHUNK_SIZE - 1)];
        let current = height_map[i];
        let lty = type_map[i.saturating_sub(1)];
        let rty = type_map[(i + 1).min(CHUNK_SIZE - 1)];
        let cty = type_map[i];
        if current != left && lty == Type::Flat && cty == Type::Flat {
            place_tile(
                fg,
                i as u32,
                left,
                Tile::Ground(GroundTileType::LedgeCap(LR::Right), set),
            )?;
        }
        if current != right && rty == Type::Flat && cty == Type::Flat {
            place_tile(
                fg,
                i as u32,
                right,
                Tile::Ground(GroundTileType::LedgeCap(LR::Left), set),
            )?;
        }
    }

    let mut runs = Vec::new();
    let mut run_start = 0;
    let mut run_height = height_map[0];
    for i in 0..CHUNK_SIZE {
        let this = height_map[i];
        if this != run_height {
            runs.push((run_start, i, run_height));
            run_start = i;
            run_height = this;
        }
    }

    for (start, end, height) in runs {
        let length = end - start;
        if length >= 10 && rng.gen_bool(0.2 + f64::clamp(length as f64 / 60.0, 0.0, 0.5)) {
            let istart = rng.gen_range(start..=(end - 2)) as u32;
            let iend = rng.gen_range(istart + 2..=istart + 4) as u32;
            let itop = rng.gen_range(height + 1 + 2..=height + 1 + u32::min(4, iend - istart));
            place_igloo(b, &mut rng, istart..iend, height + 1..itop)?;
        }
    }

    Ok(())
}
