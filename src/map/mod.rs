pub mod brushes;

use crate::{
    assets::{SpriteAssets, TILE_SIZE},
    camera::SofiaCamera,
    map::brushes::*,
};
use bevy::{
    math::{IVec2, Rect, Vec3},
    prelude::*,
};
use itertools::Itertools;
use lindel::{morton_decode, morton_encode};
use noise::{NoiseFn, Seedable};
use rand::prelude::*;
use rand::{thread_rng, SeedableRng};
use rand_pcg::Pcg64;
use std::{
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
};

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

#[derive(Component)]
pub struct Map {
    pub chunks: HashMap<Place, Chunk>,
    pub gen: Gen,
}

impl Map {
    pub fn new() -> Self {
        Map {
            chunks: HashMap::new(),
            gen: Gen::new(),
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

    fn get_changes(&self, r: Rect<f32>) -> (HashSet<Place>, HashSet<Place>) {
        let c: HashSet<Place> = HashSet::from_iter(Self::intersect(r));
        let d: HashSet<Place> = HashSet::from_iter(self.chunks.keys().copied());

        let must_make = HashSet::from_iter(c.difference(&d).copied());
        let must_delete = HashSet::from_iter(d.difference(&c).copied());
        (must_make, must_delete)
    }
}

pub struct Chunk {
    pub grid: Vec<ActiveTile>,
    pub layer: u32,
    pub place: Place,
}

impl Chunk {
    pub fn air() -> Self {
        Chunk {
            grid: vec![ActiveTile::default(); CHUNK_SIZE * CHUNK_SIZE],
            layer: 1,
            place: Place::ZERO,
        }
    }
}

impl Index<(u32, u32)> for Chunk {
    type Output = ActiveTile;
    fn index(&self, s: (u32, u32)) -> &ActiveTile {
        &self.grid[morton_encode([s.0, s.1]) as usize]
    }
}

impl IndexMut<(u32, u32)> for Chunk {
    fn index_mut(&mut self, s: (u32, u32)) -> &mut ActiveTile {
        &mut self.grid[morton_encode([s.0, s.1]) as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Component)]
pub struct ActiveTile {
    pub tile: Tile,
    pub flip: Flip,
    pub tint: Color,
    pub entity: Option<Entity>,
}

impl Default for ActiveTile {
    fn default() -> Self {
        Self {
            tile: Tile::Air,
            flip: Flip::empty(),
            tint: Color::WHITE,
            entity: None,
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
            entity: None,
        }
    }
}

fn load(
    c: &mut Chunk,
    commands: &mut Commands,
    sa: &Res<SpriteAssets>,
    coord: Place,
    parent: Entity,
) {
    for (i, tile) in c.grid.iter_mut().enumerate() {
        let layer = tile;
        if layer.tile == Tile::Air {
            continue;
        }
        let entity = layer
            .entity
            .unwrap_or_else(|| commands.spawn().insert(*layer).id());

        let [a, b]: [u32; 2] = morton_decode(i as u64);

        commands
            .entity(entity)
            //.insert(Parent(parent))
            .insert_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: layer.tint,
                    index: u16::from(layer.tile) as usize,
                    flip_x: layer.flip.contains(Flip::FLIP_H),
                    flip_y: layer.flip.contains(Flip::FLIP_V),
                },
                texture_atlas: sa.tile_texture.clone(),
                transform: Transform::from_translation(Vec3::new(
                    (a as f32 + coord.x as f32 * CHUNK_SIZE as f32) * (TILE_SIZE as f32),
                    (b as f32 + coord.y as f32 * CHUNK_SIZE as f32) * (TILE_SIZE as f32),
                    1 as f32,
                )),
                ..Default::default()
            });
    }
}

fn unload(c: &mut Chunk, commands: &mut Commands) {
    for l in c.grid.iter_mut() {
        if let Some(t) = l.entity {
            commands
                .entity(t)
                .remove_bundle::<SpriteSheetBundle>()
                .remove::<Parent>();
        }
    }
}

pub fn chunk_load_unload(
    mut commands: Commands,
    mut maps: Query<(Entity, &mut Map)>,
    sa: Res<SpriteAssets>,
    views: Query<&SofiaCamera>,
) {
    if let Some(view) = views.iter().next() {
        for (entity, mut map) in maps.iter_mut() {
            let (must_make, must_delete) = map.get_changes(view.view);
            for coord in must_make.iter() {
                let mut chunk = Chunk::air();
                chunk.place = *coord;
                gen_chunk(&mut chunk, *coord, &mut map.gen);
                load(&mut chunk, &mut commands, &sa, *coord, entity);
                map.chunks.insert(*coord, chunk);
            }

            for coord in must_delete.iter() {
                if let Some(chunk) = map.chunks.get_mut(coord) {
                    unload(chunk, &mut commands)
                }
            }
        }
    }
}

#[derive(Component)]
struct Chunk2;

fn chunk2(
    mut commands: Commands,
    chunks: Query<(Entity, &Chunk2, &Transform)>,
    sa: Res<SpriteAssets>,
    views: Query<&SofiaCamera>,
) {
    if let Some(view) = views.iter().next() {
        let mut visible: HashSet<Place> = HashSet::from_iter(Map::intersect(view.view));
        for c in chunks.iter() {
            let place = Place::new(c.2.translation.x as i32, c.2.translation.y as i32);
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
            let tiles = gen();
            for (p, t) in tiles.iter() {
                if t.tile == Tile::Air {
                    continue;
                }

                let id = commands
                    .spawn()
                    .insert_bundle(SpriteSheetBundle {
                        sprite: TextureAtlasSprite {
                            index: u16::from(t.tile) as usize,
                            ..Default::default()
                        },
                        texture_atlas: sa.tile_texture.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            p.x as f32, p.y as f32, 1 as f32,
                        )),
                        ..Default::default()
                    })
                    .id();
                children.push(id);
            }
            commands.spawn().insert(Chunk2).push_children(&children);
        }
    }
}

fn gen() -> Vec<(Place, ActiveTile)> {
    todo!()
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

fn gen_chunk(c: &mut Chunk, p: Place, g: &mut Gen) {
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
                    c[(i as u32, y)].tile = Tile::Ground(Terrain::Interior, info.theme)
                }
                if h > 0 {
                    c[(i as u32, h - 1)].tile = Tile::Ground(Terrain::Ground(LMR::M), info.theme)
                }
            }
            Feature::Hills(h) => {
                for y in 0..h {
                    c[(i as u32, y)].tile = Tile::Ground(Terrain::Interior, info.theme)
                }
                match top[i as usize] {
                    Top::TSlope(s) => {
                        c[(i as u32, h)].tile = Tile::Ground(Terrain::SlopeInt(s), info.theme);
                        if h < CHUNK_SIZE as u32 + 1 {
                            c[(i as u32, h + 1)].tile = Tile::Ground(Terrain::Slope(s), info.theme);
                        }
                    }
                    Top::TGround => {
                        c[(i as u32, h)].tile = Tile::Ground(Terrain::Ground(LMR::M), info.theme)
                    }
                    Top::TLedge(k) => {
                        c[(i as u32, h)].tile = Tile::Ground(Terrain::Ground(LMR::M), info.theme);
                        c[(i as u32, k + 1)].tile = Tile::LogLedge;
                    }
                }
            }
            Feature::ItemBoxes(h, bh, box_type) => {
                for y in 0..h {
                    c[(i as u32, y)].tile = Tile::Ground(Terrain::Interior, info.theme)
                }
                if h > 0 {
                    c[(i as u32, h - 1)].tile = Tile::Ground(Terrain::Ground(LMR::M), info.theme)
                }
                if let Some(box_type) = box_type {
                    c[(i as u32, bh)].tile = Tile::Box(box_type);
                }
            }
            Feature::Air => {}
        }
    }
}
