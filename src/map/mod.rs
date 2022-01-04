pub mod brushes;

use crate::{
    assets::{SpriteAssets, TILE_SIZE},
    camera::SofiaCamera,
    map::brushes::*,
};
use bevy::{
    math::{IVec2, Rect, Vec2, Vec3},
    prelude::*,
};
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
    rng: Pcg64,
    zone: noise::Worley,
    zone_x: noise::Fbm,
    zone_y: noise::Fbm,
    terrain: noise::OpenSimplex,
}

// must be a power of two for Morton encoding to work
// otherwise we need to change CHUNK_SIZE^2 below to to_nearest_pow2(CHUNK_SIZE)^2
pub const CHUNK_SIZE: usize = 4;

pub type Place = IVec2;

#[derive(Clone, Copy, Debug)]
pub struct Zone {
    origin: Place,
    center: Vec2,
    cell: f64,
}

impl Gen {
    pub fn new() -> Self {
        let mut tr = thread_rng();
        let mut rng = Pcg64::from_rng(&mut tr).unwrap();
        let terrain = noise::OpenSimplex::new().set_seed(rng.next_u32());
        let zone = noise::Worley::new()
            .set_seed(rng.next_u32())
            .set_range_function(noise::RangeFunction::Manhattan)
            .enable_range(true);

        let zone_x = noise::Fbm::new().set_seed(rng.next_u32());
        let zone_y = noise::Fbm::new().set_seed(rng.next_u32());

        Self {
            rng,
            zone,
            zone_x,
            zone_y,
            terrain,
        }
    }

    // fn get_zone(&self, coord: Place) -> f64 {
    //     self.zone.get([coord.x as f64, coord.y as f64])
    // }

    // fn get_zone_v(&self, coord: Place) -> Vec2 {
    //     Vec2::new(
    //         self.zone_x.get([coord.x as f64, coord.y as f64]) as f32,
    //         self.zone_y.get([coord.x as f64, coord.y as f64]) as f32,
    //     ) * (CHUNK_SIZE as f32)
    //         + Vec2::new(coord.x as f32, coord.y as f32)
    // }

    // fn zones_in_chunk(&self, coord: Place) -> [Zone; 4] {
    //     let xx = (CHUNK_SIZE as i32) * Place::X;
    //     let yy = (CHUNK_SIZE as i32) * Place::Y;
    //     let tlc = self.get_zone(coord + yy);
    //     let trc = self.get_zone(coord + yy + xx);
    //     let blc = self.get_zone(coord);
    //     let brc = self.get_zone(coord + xx);
    //     let tlz = self.get_zone_v(coord + yy);
    //     let trz = self.get_zone_v(coord + yy + xx);
    //     let blz = self.get_zone_v(coord);
    //     let brz = self.get_zone_v(coord + xx);

    //     [
    //         Zone {
    //             origin: coord + yy,
    //             center: tlz,
    //             cell: tlc,
    //         },
    //         Zone {
    //             origin: coord + yy + xx,
    //             center: trz,
    //             cell: trc,
    //         },
    //         Zone {
    //             origin: coord,
    //             center: blz,
    //             cell: blc,
    //         },
    //         Zone {
    //             origin: coord + xx,
    //             center: brz,
    //             cell: brc,
    //         },
    //     ]
    // }
}

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
            //let plan = igloo((3, 3), rng);
            let plan = pine_tree(true, 10, 10);
            run_plan(plan, c, rng, (start as u32, height + 1), 1);
        }
    }
}

#[derive(Component)]
pub struct Map {
    pub chunks: HashMap<Place, Chunk>,
    pub loaded_zones: HashSet<Place>,
    pub gen: Gen,
}

impl Map {
    pub fn new() -> Self {
        Map {
            chunks: HashMap::new(),
            loaded_zones: HashSet::new(),
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

    fn set(&mut self, coord: Place, t: Tile, l: usize) {
        let (ckx, skx) = (
            coord.x.div_euclid(CHUNK_SIZE as i32),
            coord.x.rem_euclid(CHUNK_SIZE as i32),
        );
        let (cky, sky) = (
            coord.y.div_euclid(CHUNK_SIZE as i32),
            coord.y.rem_euclid(CHUNK_SIZE as i32),
        );

        let chunk_key = Place::new(ckx, cky);
        let slot_key = Place::new(skx, sky);

        let entry = self.chunks.entry(chunk_key);
        let chunk = entry.or_insert_with(|| Chunk::air());
        chunk[(slot_key.x as u32, slot_key.y as u32)].layers[l].tile = t;
    }

    fn generate_zone(&mut self, zone: Zone) {
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
        .choose(&mut self.gen.rng)
        .unwrap();
        dbg!(zone.center);
        self.set(
            Place::new(zone.center.x as i32, zone.center.y as i32),
            Tile::Ground(Terrain::Block, set),
            1,
        );
        // let chunk_coord = Coordinate::new(zone.center.x as i32, zone.center.y as i32);
        // for i in 0..(CHUNK_SIZE as i32) {
        //     for j in 0..(CHUNK_SIZE as i32) {
        //         self.set(
        //             chunk_coord + Coordinate::new(i, j),
        //             Tile::Ground(Terrain::Block, set),
        //             1,
        //         );
        //     }
        // }
    }
}

pub struct Chunk {
    pub grid: Vec<Slot>,
}

impl Chunk {
    pub fn air() -> Self {
        Chunk {
            grid: vec![Slot::default(); CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl Index<(u32, u32)> for Chunk {
    type Output = Slot;
    fn index(&self, s: (u32, u32)) -> &Slot {
        &self.grid[morton_encode([s.0, s.1]) as usize]
    }
}

impl IndexMut<(u32, u32)> for Chunk {
    fn index_mut(&mut self, s: (u32, u32)) -> &mut Slot {
        &mut self.grid[morton_encode([s.0, s.1]) as usize]
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Slot {
    pub layers: [ActiveTile; 3],
}

impl Default for Slot {
    fn default() -> Self {
        let s = ActiveTile {
            tile: Tile::Air,
            flip: Flip::empty(),
            tint: Color::WHITE,
            entity: None,
        };
        Self { layers: [s, s, s] }
    }
}

#[derive(Clone, Copy, PartialEq, Component)]
pub struct ActiveTile {
    pub tile: Tile,
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
    c: &mut Map,
    commands: &mut Commands,
    sa: &Res<SpriteAssets>,
    coord: Place,
    parent: Entity,
) {
    if let Some(c) = c.chunks.get_mut(&coord) {
        for (i, tile) in c.grid.iter_mut().enumerate() {
            for (index, layer) in tile.layers.iter_mut().enumerate() {
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
                            (a + coord.x as u32) as f32 * (TILE_SIZE as f32),
                            (b + coord.y as u32) as f32 * (TILE_SIZE as f32),
                            index as f32,
                        )),
                        ..Default::default()
                    });
            }
        }
    }
}

fn unload(c: &mut Chunk, commands: &mut Commands) {
    for tile in c.grid.iter_mut() {
        for l in tile.layers.iter_mut() {
            if let Some(t) = l.entity {
                commands
                    .entity(t)
                    .remove_bundle::<SpriteSheetBundle>()
                    .remove::<Parent>();
            }
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
                generate_island(&mut chunk, &mut map.gen.rng);
                map.chunks.insert(*coord, chunk);
                println!("loading chunk {}", *coord);
                for zone in map.gen.zones_in_chunk(*coord) {
                    //map.generate_zone(zone);
                }
                load(
                    &mut map,
                    &mut commands,
                    &sa,
                    *coord * (CHUNK_SIZE as i32),
                    entity,
                );
            }

            for coord in must_delete.iter() {
                if let Some(mut chunk) = map.chunks.get_mut(coord) {
                    unload(chunk, &mut commands)
                }
            }
        }
    }
}
