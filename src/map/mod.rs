pub mod brushes;
pub mod level_graph;

use crate::{
    assets::{SpriteAssets, TILE_SIZE},
    camera::SofiaCamera,
    map::brushes::*,
};
use bevy::{
    math::{IVec2, Rect, Vec3},
    prelude::*,
};
use bevy_rapier2d::prelude::*;
use itertools::Itertools;
use noise::{NoiseFn, Seedable};
use rand::prelude::*;
use rand::{thread_rng, SeedableRng};
use rand_pcg::Pcg64;
use std::collections::HashSet;

pub struct Gen {
    zone: noise::Worley,
    terrain: noise::OpenSimplex,
    theme: noise::Value,
}

// must be a power of two for Morton encoding to work
// otherwise we need to change CHUNK_SIZE^2 below to to_nearest_pow2(CHUNK_SIZE)^2
pub const CHUNK_SIZE: usize = 32;

pub type Place = IVec2;

impl Gen {
    pub fn new() -> Self {
        let mut tr = thread_rng();
        let mut rng = Pcg64::from_rng(&mut tr).unwrap();
        let terrain = noise::OpenSimplex::new().set_seed(rng.next_u32());
        let zone = noise::Worley::new()
            .set_seed(rng.next_u32())
            .set_range_function(noise::RangeFunction::Manhattan)
            .enable_range(false);
        let theme = noise::Value::new().set_seed(rng.next_u32());

        Self {
            zone,
            terrain,
            theme,
        }
    }
}

pub fn intersect(r: Rect<f32>) -> impl Iterator<Item = Place> {
    let chunk_start_x = f32::floor(r.left / CHUNK_SIZE as f32) as i32;
    let chunk_end_x = f32::ceil(r.right / CHUNK_SIZE as f32) as i32;
    let chunk_start_y = f32::floor(r.bottom / CHUNK_SIZE as f32) as i32;
    let chunk_end_y = f32::ceil(r.top / CHUNK_SIZE as f32) as i32;

    itertools::iproduct!(chunk_start_x..=chunk_end_x, chunk_start_y..=chunk_end_y)
        .map(|(a, b)| Place::new(a, b))
}

#[derive(Clone, Copy, PartialEq, Component)]
pub struct ActiveTile {
    pub tile: Tile,
    pub flip: Flip,
    pub tint: Color,
}

impl Default for ActiveTile {
    fn default() -> Self {
        Self {
            tile: Tile::Air,
            flip: Flip::empty(),
            tint: Color::WHITE,
        }
    }
}

bitflags::bitflags! {
    pub struct Flip: u32 {
        const FLIP_H = 0b00000001;
        const FLIP_V = 0b00000010;
        const FLIP_D = 0b00000100;
    }
}

impl From<Tile> for ActiveTile {
    fn from(t: Tile) -> Self {
        Self {
            tile: t,
            flip: Flip::empty(),
            tint: Color::WHITE,
        }
    }
}

#[derive(Component)]
pub struct Chunk;

pub fn chunk_loader(
    mut commands: Commands,
    chunks: Query<(Entity, &Chunk, &Transform)>,
    mut res_gen: ResMut<Gen>,
    sa: Res<SpriteAssets>,
    views: Query<&SofiaCamera>,
) {
    if let Some(view) = views.iter().next() {
        let mut visible: HashSet<Place> = HashSet::from_iter(intersect(view.view));
        for c in chunks.iter() {
            let place = Place::new(
                (c.2.translation.x / (CHUNK_SIZE as f32 * TILE_SIZE as f32)) as i32,
                (c.2.translation.y / (CHUNK_SIZE as f32 * TILE_SIZE as f32)) as i32,
            );
            if visible.contains(&place) {
                visible.remove(&place);
            } else {
                // unload
                commands.entity(c.0).despawn_recursive();
            }
        }

        for p in visible.iter() {
            let mut children = Vec::new();
            // load
            let tiles = gen(*p, &mut res_gen);
            for (p, t) in tiles.iter() {
                if *t == Tile::Air {
                    continue;
                }

                let id = commands
                    .spawn()
                    // .insert_bundle(SpriteSheetBundle {
                    //     sprite: TextureAtlasSprite {
                    //         index: u16::from(*t) as usize,
                    //         ..Default::default()
                    //     },
                    //     texture_atlas: sa.tile_texture.clone(),
                    //     transform: Transform::from_translation(Vec3::new(
                    //         p.x as f32 * TILE_SIZE as f32,
                    //         p.y as f32 * TILE_SIZE as f32,
                    //         1 as f32,
                    //     )),
                    //     ..Default::default()
                    // })
                    .insert_bundle(ColliderBundle {
                        shape: ColliderShapeComponent(ColliderShape::cuboid(0.5, 0.5)),
                        position: ColliderPositionComponent(ColliderPosition(
                            Isometry::translation(p.x as f32, p.y as f32),
                        )),
                        ..Default::default()
                    })
                    .insert(ColliderDebugRender::with_id(3))
                    .id();
                children.push(id);
            }
            commands
                .spawn()
                .insert(Chunk)
                .insert(Transform::from_translation(Vec3::new(
                    p.x as f32 * CHUNK_SIZE as f32 * TILE_SIZE as f32,
                    p.y as f32 * CHUNK_SIZE as f32 * TILE_SIZE as f32,
                    1.0,
                )))
                .insert(GlobalTransform::default())
                .push_children(&children);
        }
    }
}

fn gen(p: Place, g: &mut Gen) -> Vec<(Place, Tile)> {
    let mut c = Vec::new();
    let mut kinds = Vec::new();
    for i in -1..(CHUNK_SIZE as i32 + 1) {
        kinds.push(sample_kind(p * CHUNK_SIZE as i32 + i * Place::X, g));
    }

    #[derive(Clone, Copy, Debug)]
    enum Top {
        TSlope(Slope),
        TGround,
        TLedge(u32),
    }

    let mut top = Vec::new();
    for (x0, x1, x2) in kinds.iter().copied().tuple_windows() {
        let x0 = u32::from(x0.feature);
        let x1 = u32::from(x1.feature);
        let x2 = u32::from(x2.feature);

        use Top::*;
        if x0 == x2 && x0 > x1 {
            top.push(TLedge(x0));
        } else if x0 == x1 + 1 {
            top.push(TSlope(Slope::DownLeft));
        } else if x1 + 1 == x2 {
            top.push(TSlope(Slope::UpRight));
        } else {
            top.push(TGround)
        }
    }

    for (i, info) in kinds[1..=CHUNK_SIZE].iter().copied().enumerate() {
        match info.feature {
            Feature::Run(h) => {
                for y in 0..h {
                    c.push((
                        Place::new(i as i32, y as i32),
                        Tile::Ground(Terrain::Interior, info.theme),
                    ));
                }
                if h > 0 {
                    c.push((
                        Place::new(i as i32, h as i32),
                        Tile::Ground(Terrain::Ground(LMR::M), info.theme),
                    ));
                }
            }
            Feature::Hills(h) => {
                for y in 0..h {
                    c.push((
                        Place::new(i as i32, y as i32),
                        Tile::Ground(Terrain::Interior, info.theme),
                    ));
                }
                match top[i as usize] {
                    Top::TSlope(s) => {
                        c.push((
                            Place::new(i as i32, h as i32),
                            Tile::Ground(Terrain::SlopeInt(s), info.theme),
                        ));
                        if h < CHUNK_SIZE as u32 + 1 {
                            c.push((
                                Place::new(i as i32, (h + 1) as i32),
                                Tile::Ground(Terrain::Slope(s), info.theme),
                            ));
                        }
                    }
                    Top::TGround => {
                        c.push((
                            Place::new(i as i32, h as i32),
                            Tile::Ground(Terrain::Ground(LMR::M), info.theme),
                        ));
                    }
                    Top::TLedge(k) => {
                        c.push((
                            Place::new(i as i32, h as i32),
                            Tile::Ground(Terrain::Ground(LMR::M), info.theme),
                        ));
                        c.push((Place::new(i as i32, (k + 1) as i32), Tile::LogLedge));
                    }
                }
            }
            Feature::ItemBoxes(h, bh, box_type) => {
                for y in 0..h {
                    c.push((
                        Place::new(i as i32, y as i32),
                        Tile::Ground(Terrain::Interior, info.theme),
                    ));
                }
                if h > 0 {
                    c.push((
                        Place::new(i as i32, h as i32),
                        Tile::Ground(Terrain::Ground(LMR::M), info.theme),
                    ));
                }
                if let Some(box_type) = box_type {
                    c.push((Place::new(i as i32, bh as i32), Tile::Box(box_type)));
                }
            }
            Feature::Air => {}
        }
    }

    c
}

#[derive(Clone, Copy, Debug)]
enum Feature {
    Run(u32),
    Hills(u32),
    ItemBoxes(u32, u32, Option<BoxPiece>),
    Air,
}

impl From<Feature> for u32 {
    fn from(f: Feature) -> u32 {
        match f {
            Feature::Run(h) => h,
            Feature::Hills(h) => h,
            Feature::ItemBoxes(h, _, _) => h,
            Feature::Air => 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Kind {
    theme: TerrainTheme,
    feature: Feature,
}

fn sample_basic_height(p: Place, g: &mut Gen) -> u32 {
    (12.0 * g.terrain.get([p.x as f64 * 0.04, 0.0]).abs() + 1.0) as u32
}

fn sample_zone_center(p: Place, g: &mut Gen) -> Place {
    let range = g.zone.get([p.x as f64 * 0.1, p.y as f64 * 0.1]);
    let left = g.zone.get([p.x as f64 * 0.1 - range, p.y as f64 * 0.1]);
    let right = g.zone.get([p.x as f64 * 0.1 + range, p.y as f64 * 0.1]);
    let sign = if left > right { -1.0 } else { 1.0 };
    let zone_start = p.x as f64 + sign * range;
    Place::new(zone_start as i32, p.y)
}

fn sample_thematic(p: Place, g: &mut Gen) -> f64 {
    let rate = 0.008;
    g.theme.get([p.x as f64 * rate, p.y as f64 * rate])
}

fn sample_random(p: Place, g: &mut Gen) -> f64 {
    let rate = 1.0;
    g.theme.get([p.x as f64 * rate, p.y as f64 * rate])
}

fn sample_theme(p: Place, g: &mut Gen) -> TerrainTheme {
    let theme = sample_thematic(p, g);
    [
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
    ][((theme + 1.0) / 2.0 * 11.0) as usize]
}

fn sample_kind(p: Place, g: &mut Gen) -> Kind {
    if p.y != 0 {
        return Kind {
            theme: TerrainTheme::Grass,
            feature: Feature::Air,
        };
    }

    let zc = sample_zone_center(p, g);
    let theme = sample_theme(zc, g);
    let height = sample_basic_height(zc, g);

    let s = ((sample_thematic(zc, g).abs() * 3.0) % 3.0) as u32;

    if s == 0 {
        Kind {
            theme,
            feature: Feature::Run(height as u32),
        }
    } else if s == 1 {
        Kind {
            theme,
            feature: Feature::Hills(height as u32),
        }
    } else {
        let k = ((sample_random(zc, g).abs() * 10.0) % 10.0) as u32;
        let bt = match k {
            0 => Some(BoxPiece::Item { used: false }),
            1 => Some(BoxPiece::Coin { used: false }),
            2 | 3 => Some(BoxPiece::Crate),
            _ => None,
        };

        Kind {
            theme,
            feature: Feature::ItemBoxes(height as u32, height as u32 + 4, bt),
        }
    }
}
