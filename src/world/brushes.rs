use bevy::prelude::Resource;
use enum_iterator::Sequence;
use itertools::iproduct;
use itertools::Itertools;
use itertools::Position;
use noise::NoiseFn;
use num_derive::FromPrimitive;
use rand::thread_rng;
use rand::Rng;
use ranges::GenericRange;
use ranges::Ranges;
use std::ops::Bound;
use std::ops::RangeBounds;

use crate::helpers::*;

use super::{feature::*, tile::*, Place};

#[derive(Resource)]
pub struct Gen {
    pub terrain: noise::ScaleBias<f64, noise::Fbm<noise::SuperSimplex>, 2>,
    pub zone: noise::ScaleBias<f64, noise::SuperSimplex, 2>,
    pub theme: noise::ScaleBias<f64, noise::Value, 2>,
    pub seed: u32,
}

impl Default for Gen {
    fn default() -> Self {
        let mut tr = thread_rng();
        let seed: u32 = tr.gen();

        use noise::MultiFractal;

        let octaves_n = 4;
        let octaves = (seed..)
            .take(octaves_n)
            .map(|i| noise::SuperSimplex::new(i))
            .collect();
        let p = 0.2;
        let terrain = noise::ScaleBias::new(
            noise::Fbm::new(seed)
                .set_sources(octaves)
                .set_octaves(octaves_n)
                .set_frequency(1.0 / 32.0)
                .set_lacunarity(2.0)
                .set_persistence(p),
        )
        .set_scale(1.0 / (2.0 * (1.0 + p + p * p + p * p * p)))
        .set_bias(0.5);
        let zone = noise::ScaleBias::new(noise::SuperSimplex::new(seed + octaves_n as u32))
            .set_scale(0.5)
            .set_bias(0.5);
        let theme = noise::ScaleBias::new(noise::Value::new(seed + octaves_n as u32))
            .set_scale(0.5)
            .set_bias(0.5);

        Self {
            zone,
            terrain,
            theme,
            seed,
        }
    }
}

pub fn generate_level(gen: &Gen) -> Schema {
    let mut schema = Schema::default();

    let height = 10;

    let size = 1000;
    let mut x = -size;
    while x < size {
        let z = n_to_enum(gen.zone.get([x as f64, 0.0]));
        let w = gen.zone.get([x as f64, 2.0]) * 10.0 % 1.0;
        let w = (50.0 * w + 20.0) as i32;
        let b = Box2::from_box1s(Box1::new(x, x + w), Box1::new(-height, height + 1));

        schema.add(Feature::Zone(z, b));
        height_map_floor_brush(&mut schema, gen, z, b);
        match z {
            Zone::Grass(Zone1::Lake) | Zone::Desert(Zone1::Lake) | Zone::Candy(Zone1::Lake) => {
                liquid_brush(&mut schema, gen, b, Liquid::Water)
            }
            Zone::LavaHills | Zone::LavaPlains => liquid_brush(&mut schema, gen, b, Liquid::Lava),
            Zone::Mushroom => {
                big_mushroom_brush(&mut schema, gen, b);
                mushroom_brush(&mut schema, gen, b);
            }
            Zone::Caverns => {
                cavern_roof_brush(&mut schema, gen, b);
                cavern_tunnel_brush(&mut schema, gen, b);
            }
            Zone::Forest => tree_brush(&mut schema, gen, b, false),
            Zone::SnowForest => tree_brush(&mut schema, gen, b, true),
            Zone::Grass(_)
            | Zone::Desert(_)
            | Zone::Candy(_)
            | Zone::StoneMountain
            | Zone::StoneCliff
            | Zone::Castle => (),
        }
        bonus_brush(&mut schema, gen, b);
        x += w;
    }

    schema
}

fn height_map_floor_brush(schema: &mut Schema, gen: &Gen, zone: Zone, region: Box2<i32>) {
    let runs = region
        .x
        .iter()
        .map(|i| {
            let height = n_to_box1(gen.terrain.get([i as f64, 0.0]), region.y);
            let gap = gen.zone.get([i as f64, 0.0]) < zone.info().gap_chance;
            (1, i, if gap { region.y.lo_incl } else { height })
        })
        .coalesce(|(l1, x1, a), (l2, x2, b)| {
            if a == b {
                Ok((l1 + l2, x1, a))
            } else {
                Err(((l1, x1, a), (l2, x2, b)))
            }
        });

    for pos in runs.tuple_windows().with_position() {
        let ((l1, x1, h1), (l2, x2, h2)) = pos.into_inner();
        assert!(h1 != h2);
        assert!(l1 > 0 && l2 > 0);
        assert!(x1 < x2);
        assert!(x1 + l1 == x2);
        assert!(h1 >= region.y.lo_incl && h2 >= region.y.lo_incl);

        let is_hill = gen.zone.get([x1 as f64, 1.0]) < zone.info().hill_chance;

        let run_length = (h2 - h1).abs();
        let x_run = n_to_fitted_box1(
            gen.terrain.get([x1 as f64, 5.0]),
            run_length,
            Box1::new(x1, x2),
        );

        if is_hill && h1 > region.y.lo_incl && let Some(x_run) = x_run {
            let max_thickness = (h1.min(h2) - region.y.lo_incl) as u32;
            let bridge_thickness =
                f64::max(0.0, gen.zone.get([x1 as f64, 2.0]) * 30.0 - 22.0).floor();
            let bridge_thickness = (bridge_thickness as u32).min(max_thickness);
            let bridge_thickness = if bridge_thickness < 2 {
                None
            } else {
                Some(bridge_thickness + 2)
            };
            let (lr, height) = if h1 < h2 {
                (LR::L, Box1::new(h1, h2))
            } else {
                (LR::R, Box1::new(h2, h1))
            };
            schema.add(Feature::HillBlock {
                terrain: zone.info().terrain,
                start_x: x_run.lo_incl,
                height,
                bridge_thickness,
                lr,
            });
            if x1 < x_run.lo_incl {
                assert!(h1 > region.y.lo_incl);
                schema.add(Feature::GroundBlock(
                    GroundCover::TopCovered,
                    zone.info().terrain,
                    Box2 {
                        x: Box1::new(x1, x_run.lo_incl),
                        y: Box1::new(region.y.lo_incl, h1),
                    },
                ));
            }
            if x_run.hi_excl < x2 && h2 > region.y.lo_incl {
                schema.add(Feature::GroundBlock(
                    GroundCover::TopCovered,
                    zone.info().terrain,
                    Box2 {
                        x: Box1::new(x_run.hi_excl, x2),
                        y: Box1::new(region.y.lo_incl, h2),
                    },
                ));
            }
            if bridge_thickness.is_none() && h2 > region.y.lo_incl {
                schema.add(Feature::GroundBlock(
                    GroundCover::TopCovered,
                    zone.info().terrain,
                    Box2 {
                        x: x_run,
                        y: Box1::new(region.y.lo_incl, h1.min(h2)),
                    },
                ));
            }
        } else if h1 > region.y.lo_incl {
            schema.add(Feature::GroundBlock(
                GroundCover::TopCovered,
                zone.info().terrain,
                Box2 {
                    x: Box1::new(x1, x1 + l1),
                    y: Box1::new(region.y.lo_incl, h1),
                },
            ));
        }

        if let Position::Last(_) = pos && h2 > region.y.lo_incl {
            schema.add(Feature::GroundBlock(
                GroundCover::TopCovered,
                zone.info().terrain,
                Box2 {
                    x: Box1::new(x2, x2 + l2),
                    y: Box1::new(region.y.lo_incl, h2),
                },
            ));
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Sequence, FromPrimitive)]
enum Liquid {
    Water,
    Lava,
}

fn liquid_brush(schema: &mut Schema, gen: &Gen, box2: Box2<i32>, liquid: Liquid) {
    let mut height = box2.y.hi_excl;
    for f in schema.intersecting(box2) {
        match f {
            Feature::FlatGround(p, _) if p.y > box2.y.lo_incl => height = height.min(p.y),
            Feature::SlopedGround { start, height: h } => {
                let min = start.y + 0.max(h);
                if min > box2.y.lo_incl {
                    height = height.min(min);
                }
            }
            _ => (),
        }
    }
    let height = n_to_box1(
        gen.terrain.get([box2.x.lo_incl as f64, 0.0]),
        Box1::new(box2.y.lo_incl + 1, height + 2),
    );
    let b = Box2::from_box1s(box2.x, Box1::new(box2.y.lo_incl, height));
    match liquid {
        Liquid::Water => schema.add(Feature::SurfaceWater(b)),
        Liquid::Lava => schema.add(Feature::SurfaceLava(b)),
    }
}

fn big_mushroom_brush(schema: &mut Schema, gen: &Gen, box2: Box2<i32>) {
    let mut open_ranges = Ranges::new();
    open_ranges.insert(GenericRange::new_closed_open(
        box2.x.lo_incl,
        box2.x.hi_excl,
    ));
    for f in schema.intersecting(box2) {
        match f {
            Feature::FlatGround(p, width) => {
                open_ranges.remove(GenericRange::new_closed_open(p.x, p.x + width as i32));
            }
            Feature::SlopedGround { start, height } => {
                open_ranges.remove(GenericRange::new_closed_open(
                    start.x,
                    start.x + height.abs(),
                ));
            }
            _ => (),
        }
    }

    let mut ranges = open_ranges
        .as_slice()
        .iter()
        .map(|&gr| {
            let start = match gr.start_bound() {
                Bound::Included(x) => *x,
                Bound::Excluded(x) => *x + 1,
                Bound::Unbounded => panic!("shouldn't be an unbounded bound"),
            };
            let end = match gr.end_bound() {
                Bound::Included(x) => *x + 1,
                Bound::Excluded(x) => *x,
                Bound::Unbounded => panic!("shouldn't be an unbounded bound"),
            };
            Box1::new(start, end)
        })
        .collect::<Vec<_>>();
    ranges.sort_by(|r1, r2| r1.size().cmp(&r2.size()).reverse());

    // TERMINATION: loops over an ever-shrinking set of ranges
    while let Some(b) = ranges.pop() {
        if b.size() < 11 {
            break;
        }
        let n = gen.terrain.get([b.lo_incl as f64, 2.0]);
        let m = gen.terrain.get([b.lo_incl as f64, 3.0]);
        let size = n_to_box1(n, Box1::new(1, b.size() / 2));
        let cap = n_to_fitted_box1(m, 2 * size + 1, b);
        if let Some(cap) = cap {
            if cap.lo_incl > b.lo_incl {
                ranges.push(Box1::new(b.lo_incl, b.hi_excl.min(cap.lo_incl)));
            }
            if cap.hi_excl < b.hi_excl {
                ranges.push(Box1::new(b.lo_incl.max(cap.hi_excl), b.hi_excl));
            }
            let height = n_to_box1(gen.terrain.get([b.lo_incl as f64, 0.0]), box2.y);
            schema.add(Feature::BigMushroomTop(
                Place::new(cap.lo_incl + size, height),
                size as u32,
            ));
            schema.add(Feature::BigMushroomStem(
                Place::new(cap.lo_incl + size, box2.y.lo_incl),
                (height - box2.y.lo_incl) as u32,
            ));
        }
    }
}

fn mushroom_brush(_schema: &mut Schema, _gen: &Gen, _box2: Box2<i32>) {
    // todo
}

fn cavern_roof_brush(_schema: &mut Schema, _gen: &Gen, _box2: Box2<i32>) {
    // todo
}

fn cavern_tunnel_brush(_schema: &mut Schema, _gen: &Gen, _box2: Box2<i32>) {
    // todo
}

fn tree_brush(_schema: &mut Schema, _gen: &Gen, _box2: Box2<i32>, _snow: bool) {
    // todo
}

fn bonus_brush(_schema: &mut Schema, _gen: &Gen, _box2: Box2<i32>) {
    // todo
}

#[derive(Clone, Copy, Debug)]
struct LayeredTile {
    background: TilingTile,
    midground: TilingTile,
    foreground: TilingTile,
}

fn get_tile(schema: &Schema, gen: &Gen, p: Place) -> LayeredTile {
    let mut t = LayeredTile {
        background: TilingTile::Exactly(Tile::Air),
        midground: TilingTile::Exactly(Tile::Air),
        foreground: TilingTile::Exactly(Tile::Air),
    };

    let altn = gen.theme.get([p.x as f64, p.y as f64]);

    for f in schema.at_point(p) {
        match f {
            Feature::GroundBlock(gc, terrain, _) => t.midground = TilingTile::Ground(gc, terrain),
            Feature::HillBlock {
                terrain,
                start_x,
                height,
                bridge_thickness,
                lr,
            } => {
                let top = match lr {
                    LR::L => height.lo_incl + (p.x - start_x),
                    LR::R => height.hi_excl - 1 - (p.x - start_x),
                };
                match bridge_thickness {
                    None => {
                        if p.y == top {
                            t.midground =
                                TilingTile::Exactly(Tile::Terrain(terrain, TerrainTile::Slope(lr)));
                        } else if p.y < top {
                            t.midground = TilingTile::Ground(GroundCover::TopCovered, terrain);
                        }
                    }
                    Some(bridge_thickness) => {
                        let bottom = top - bridge_thickness as i32;
                        if p.y == top {
                            t.midground =
                                TilingTile::Exactly(Tile::Terrain(terrain, TerrainTile::Slope(lr)));
                        } else if bottom < p.y && p.y < top {
                            t.midground = TilingTile::Ground(GroundCover::TopCovered, terrain)
                        } else if p.y == bottom {
                            t.midground = TilingTile::Exactly(Tile::Terrain(
                                terrain,
                                TerrainTile::RockSlope(lr.flip(), TB::B),
                            ));
                        }
                    }
                }
            }
            Feature::Igloo { box2, door } => {
                let lmr = lmr_of(box2.x, p.x);
                let tmb = tmb_of(box2.y, p.y);
                if tmb == TMB::B && box2.x.lo_incl + door as i32 == p.x {
                    t.background = TilingTile::Exactly(Tile::IglooDoor);
                } else if tmb == TMB::T {
                    t.background = TilingTile::Exactly(Tile::IglooTop(lmr));
                } else {
                    t.background = TilingTile::Exactly(Tile::IglooInterior(n_to_bool(altn)));
                }
            }
            Feature::Tile(_, tile) => t.foreground = TilingTile::Exactly(tile),
            Feature::CrateCrossRect(_) => t.midground = TilingTile::Exactly(Tile::CrateCross),
            Feature::CrateRandomRect(_) => {
                t.midground = TilingTile::Exactly(
                    [Tile::CrateBlank, Tile::CrateSlash, Tile::CrateCross][n_to_range(altn, 3)],
                )
            }
            Feature::SurfaceWater(b) => {
                if p.y == b.y.hi_excl - 1 {
                    t.background = TilingTile::Exactly(Tile::WaterWave);
                } else {
                    t.background = TilingTile::Exactly(Tile::Water);
                }
            }
            Feature::SurfaceLava(b) => {
                if p.y == b.y.hi_excl - 1 {
                    t.background = TilingTile::Exactly(Tile::LavaWave);
                } else {
                    t.background = TilingTile::Exactly(Tile::Lava);
                }
            }
            Feature::BigMushroomTop(center, width) => {
                let style = n_to_enum(gen.theme.get([center.x as f64, center.y as f64]));
                let alt = n_to_bool(altn);
                if p.x == center.x {
                    t.midground = TilingTile::Exactly(Tile::MushroomStemBlock(style, alt));
                } else if p.x == center.x - width as i32 {
                    t.midground = TilingTile::Exactly(Tile::MushroomBlock(style, alt, LMR::L));
                } else if p.x == center.x + width as i32 {
                    t.midground = TilingTile::Exactly(Tile::MushroomBlock(style, alt, LMR::R));
                } else {
                    t.midground = TilingTile::Exactly(Tile::MushroomBlock(style, alt, LMR::M));
                }
            }
            Feature::BigMushroomStem(base, height) => {
                if p.y == base.y + height as i32 {
                    t.background = TilingTile::Exactly(Tile::MushroomStemTop(n_to_bool(altn)));
                } else {
                    t.background = TilingTile::Exactly(
                        [
                            Tile::MushroomStem,
                            Tile::MushroomStemLeaf,
                            Tile::MushroomStemRing(false),
                            Tile::MushroomStemRing(true),
                        ][n_to_range(altn, 4)],
                    )
                }
            }

            Feature::SlopedGround { .. }
            | Feature::FlatGround(_, _)
            | Feature::Zone(_, _)
            | Feature::Offscreen(_) => (),
        }
    }

    t
}

pub fn render_level(schema: &Schema, gen: &Gen, box2: Box2<i32>) -> ndarray::Array3<Tile> {
    let extended = Box2 {
        x: Box1::new(box2.x.lo_incl - 1, box2.x.hi_excl + 1),
        y: Box1::new(box2.y.lo_incl - 1, box2.y.hi_excl + 1),
    };
    let shape = [box2.x.size() as usize, box2.y.size() as usize];
    let shape_ex = [extended.x.size() as usize, extended.y.size() as usize];
    let mut back = ndarray::Array::from_elem(shape_ex, TilingTile::Exactly(Tile::Air));
    let mut mid = ndarray::Array::from_elem(shape_ex, TilingTile::Exactly(Tile::Air));
    let mut fore = ndarray::Array::from_elem(shape_ex, TilingTile::Exactly(Tile::Air));

    for (i, j) in iproduct!(extended.x.iter(), extended.y.iter()) {
        let i_ = (i - extended.x.lo_incl) as usize;
        let j_ = (j - extended.y.lo_incl) as usize;

        let LayeredTile {
            background,
            midground,
            foreground,
        } = get_tile(schema, gen, Place::new(i, j));
        back[[i_, j_]] = background;
        mid[[i_, j_]] = midground;
        fore[[i_, j_]] = foreground;
    }

    let mid2 = compute_tiling(mid);

    let mut array = ndarray::Array::from_elem([shape[0], shape[1], 5], Tile::Air);

    for (i, j) in iproduct!(box2.x.iter(), box2.y.iter()) {
        let i_ = (i - box2.x.lo_incl) as usize;
        let j_ = (j - box2.y.lo_incl) as usize;
        if let TilingTile::Exactly(t) = back[[i_ + 1, j_ + 1]] {
            array[[i_, j_, 0]] = t;
        }
        array[[i_, j_, 1]] = mid2[[i_, j_]].0;
        if let TilingTile::Exactly(t) = fore[[i_ + 1, j_ + 1]] {
            array[[i_, j_, 2]] = t;
        }
        if let Some(terrain) = mid2[[i_, j_]].1.left_cap {
            array[[i_, j_, 3]] = Tile::Terrain(terrain, TerrainTile::Cap(LR::L));
        }
        if let Some(terrain) = mid2[[i_, j_]].1.right_cap {
            array[[i_, j_, 4]] = Tile::Terrain(terrain, TerrainTile::Cap(LR::R));
        }
    }
    array
}
