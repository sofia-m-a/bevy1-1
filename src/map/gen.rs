use extent::Extent;
use noise::{NoiseFn, Seedable};
use rand::prelude::*;
use rand_pcg::Pcg64;

use crate::{map::brushes::*, map::map::Chunk, map::map::CHUNK_SIZE};

pub fn generate_island(c: &mut Chunk, rng: &mut Pcg64) {
    let noise = noise::OpenSimplex::new().set_seed(rng.next_u32());

    let set = *[
        TerrainTheme::Grass,
        TerrainTheme::Dirt,
        TerrainTheme::Sand,
        TerrainTheme::Stone,
        TerrainTheme::Castle,
        TerrainTheme::Metal,
        TerrainTheme::Stone,
        TerrainTheme::Snow,
        TerrainTheme::Tundra,
        TerrainTheme::Cake,
        TerrainTheme::Choco,
    ]
    .choose(rng)
    .unwrap();

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
                Type::Slope(s) if top => Terrain::Slope(s),
                Type::Slope(s) if top1 => Terrain::SlopeInt(s),
                Type::Flat if top => Terrain::Ground(LMR::M),
                _ => Terrain::Interior,
            };

            c[(i as u32, j)].layers[1] = Tile::Ground(tile, set).into();
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
            c[(i as u32, left)].layers[2] = Tile::Ground(Terrain::LedgeCap(LR::R), set).into();
        }
        if current != right && rty == Type::Flat && cty == Type::Flat {
            c[(i as u32, right)].layers[2] = Tile::Ground(Terrain::LedgeCap(LR::L), set).into();
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
            let plan = igloo((3, 3), rng);
            run_plan(plan, c, rng, (start as u32, height + 1), 1);
        }
    }
}
