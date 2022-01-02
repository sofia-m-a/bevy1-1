use std::ops::Range;

use noise::{NoiseFn, Seedable};
use rand::prelude::*;
use rand_pcg::Pcg64;

use crate::{
    map::brushes::{GroundSet, GroundTileType, IglooPiece, Slope, TileType, LMR, LR},
    map::map::Chunk,
    map::map::CHUNK_SIZE,
};

fn place_igloo(c: &mut Chunk, rng: &mut Pcg64, width: Range<u32>, height: Range<u32>) {
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
            c[(i, j)].layers[1] = TileType::Igloo(piece).into();
        }
    }

    if height.len() >= 2 {
        let door_location = rng.gen_range(width);
        c[(door_location, height.start)].layers[1] = TileType::Igloo(IglooPiece::Door).into();
    }
}

pub fn generate_island(c: &mut Chunk) {
    let mut tr = thread_rng();
    let mut rng = Pcg64::from_rng(&mut tr).unwrap();
    let noise = noise::OpenSimplex::new().set_seed(rng.next_u32());

    let set = *[
        GroundSet::Grass,
        GroundSet::Dirt,
        GroundSet::Sand,
        GroundSet::Stone,
        GroundSet::Castle,
        GroundSet::Metal,
        GroundSet::Stone,
        GroundSet::Snow,
        GroundSet::Tundra,
        GroundSet::Cake,
        GroundSet::Choco,
    ]
    .choose(&mut rng)
    .unwrap();

    let mut height_map = [0; CHUNK_SIZE];

    for i in 0..CHUNK_SIZE {
        let height = (12.0 * noise.get([(i as f64) * 0.06, i as f64 * 0.06]).abs()) as u32;
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

            c[(i as u32, j)].layers[1] = TileType::Ground(tile, set).into();
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
            c[(i as u32, left)].layers[2] =
                TileType::Ground(GroundTileType::LedgeCap(LR::Right), set).into();
        }
        if current != right && rty == Type::Flat && cty == Type::Flat {
            c[(i as u32, right)].layers[2] =
                TileType::Ground(GroundTileType::LedgeCap(LR::Left), set).into();
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
            place_igloo(c, &mut rng, istart..iend, height + 1..itop);
        }
    }
}
