use enum_iterator::Sequence;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use rstar::iterators::LocateAllAtPoint;
use rstar::iterators::LocateInEnvelopeIntersecting;
use rstar::*;
use std::iter::Copied;

use super::{tile::*, Place};
use crate::helpers::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Sequence, FromPrimitive)]
pub enum Zone1 {
    Plains,
    Hills,
    Lake,
    Sky,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Sequence)]
pub enum Zone {
    Grass(Zone1),
    Desert(Zone1),
    Candy(Zone1),

    Mushroom,
    Caverns,
    Forest,
    SnowForest,
    StoneMountain,
    StoneCliff,
    LavaPlains,
    LavaHills,
    Castle,
}

pub struct ZoneInfo {
    pub gap_chance: f64,
    pub hill_chance: f64,
    pub terrain: Terrain,
    pub alt_terrain: Option<Terrain>,
}

impl Zone {
    pub fn info(self) -> ZoneInfo {
        let (gap_chance, hill_chance) = match self {
            Zone::Grass(z1) | Zone::Desert(z1) | Zone::Candy(z1) => match z1 {
                Zone1::Plains => (0.1, 0.1),
                Zone1::Hills => (0.2, 0.9),
                Zone1::Lake => (0.4, 0.3),
                Zone1::Sky => (0.6, 0.0),
            },
            Zone::Mushroom => (0.8, 0.0),
            Zone::Caverns => (0.2, 0.6),
            Zone::Forest | Zone::SnowForest => (0.1, 0.3),
            Zone::StoneMountain => (0.3, 0.3),
            Zone::StoneCliff => (0.3, 0.1),
            Zone::LavaPlains => (0.2, 0.1),
            Zone::LavaHills => (0.3, 0.9),
            Zone::Castle => (0.3, 0.2),
        };

        let (terrain, alt_terrain) = match self {
            Zone::Grass(_) | Zone::Forest => (Terrain::Grass, None),
            Zone::Desert(_) => (Terrain::Sand, None),
            Zone::Candy(_) => (Terrain::Cake, Some(Terrain::Choco)),
            Zone::Mushroom => (Terrain::Dirt, Some(Terrain::Grass)),
            Zone::Caverns => (Terrain::Stone, None),
            Zone::SnowForest => (Terrain::Snow, None),
            Zone::StoneMountain | Zone::StoneCliff => (Terrain::Stone, Some(Terrain::Dirt)),
            Zone::LavaPlains | Zone::LavaHills => (Terrain::Stone, None),
            Zone::Castle => (Terrain::Castle, None),
        };

        ZoneInfo {
            gap_chance,
            hill_chance,
            terrain,
            alt_terrain,
        }
    }
}

impl FromPrimitive for Zone {
    fn from_i64(n: i64) -> Option<Self> {
        Self::from_u64(u64::try_from(n).ok()?)
    }

    fn from_u64(n: u64) -> Option<Self> {
        pub const Z1: u64 = Zone1::CARDINALITY as u64;
        match n {
            0 => Some(Zone::Mushroom),
            1 => Some(Zone::Caverns),
            2 => Some(Zone::Forest),
            3 => Some(Zone::SnowForest),
            4 => Some(Zone::StoneMountain),
            5 => Some(Zone::StoneCliff),
            6 => Some(Zone::LavaPlains),
            7 => Some(Zone::LavaHills),
            8 => Some(Zone::Castle),

            n if n < 9 + 1 * Z1 => Some(Zone::Grass(Zone1::from_u64((n - 8) % Z1)?)),
            n if n < 9 + 2 * Z1 => Some(Zone::Desert(Zone1::from_u64((n - 8) % Z1)?)),
            n if n < 9 + 3 * Z1 => Some(Zone::Candy(Zone1::from_u64((n - 8) % Z1)?)),

            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Feature {
    GroundBlock(GroundCover, Terrain, Box2<i32>),
    HillBlock {
        terrain: Terrain,
        start_x: i32,
        height: Box1<i32>,
        bridge_thickness: Option<u32>,
        lr: LR,
    },
    Igloo {
        box2: Box2<i32>,
        door: u32,
    },
    Tile(Place, Tile),
    CrateCrossRect(Box2<i32>),
    CrateRandomRect(Box2<i32>),
    SurfaceWater(Box2<i32>),
    SurfaceLava(Box2<i32>),
    BigMushroomTop(Place, u32),
    BigMushroomStem(Place, u32),

    SlopedGround {
        start: Place,
        height: i32,
    },
    FlatGround(Place, u32),

    Zone(Zone, Box2<i32>),
    Offscreen(Box2<i32>),
}

impl Feature {
    pub fn describe(self) -> &'static str {
        match self {
            Feature::GroundBlock(_, _, _) => "Ground",
            Feature::HillBlock {
                bridge_thickness: None,
                ..
            } => "Hill",
            Feature::HillBlock {
                bridge_thickness: Some(_),
                ..
            } => "Hill bridge",
            Feature::Igloo { .. } => "Igloo",
            Feature::Tile(_, _) => "Tile",
            Feature::CrateCrossRect(_) => "Cross crates",
            Feature::CrateRandomRect(_) => "Random crates",
            Feature::SurfaceWater(_) => "Surface water",
            Feature::SurfaceLava(_) => "Surface lava",
            Feature::BigMushroomTop(_, _) => "Mushroom top",
            Feature::BigMushroomStem(_, _) => "Mushroom stem",
            Feature::SlopedGround { .. } => "Sloped ground",
            Feature::FlatGround(_, _) => "Flat ground",
            Feature::Zone(_, _) => "Zone",
            Feature::Offscreen(_) => "Offscreen",
        }
    }

    pub fn bounds(self) -> Box2<i32> {
        match self {
            Feature::GroundBlock(_, _, b) => b,
            Feature::HillBlock {
                start_x,
                height,
                bridge_thickness,
                ..
            } => Box2::new(
                (
                    start_x,
                    height.lo_incl - bridge_thickness.unwrap_or(0) as i32,
                ),
                (start_x + height.size(), height.hi_excl),
            ),
            Feature::Igloo { box2, .. } => box2,
            Feature::Tile(p, _) => Box2::from_point(p.into()),
            Feature::CrateCrossRect(b) => b,
            Feature::CrateRandomRect(b) => b,
            Feature::SurfaceWater(b) => b,
            Feature::SurfaceLava(b) => b,
            Feature::BigMushroomTop(p, width) => Box2::from_box1s(
                Box1::new(p.x - width as i32, p.x + width as i32 + 1),
                Box1::from_point(p.y),
            ),
            Feature::BigMushroomStem(p, height) => {
                Box2::from_box1s(Box1::from_point(p.x), Box1::new(p.y, p.y + height as i32))
            }
            Feature::SlopedGround { start, height } => Box2::new(
                (start.x, start.y),
                (start.x + height.abs(), start.y + height.abs()),
            ),
            Feature::FlatGround(p, length) => {
                Box2::from_box1s(Box1::new(p.x, p.x + length as i32), Box1::from_point(p.y))
            }
            Feature::Zone(_, b) => b,
            Feature::Offscreen(b) => b,
        }
    }
}

impl RTreeObject for Feature {
    type Envelope = AABB<(i32, i32)>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds().into()
    }
}

impl PointDistance for Feature {
    fn distance_2(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
    ) -> <<Self::Envelope as Envelope>::Point as Point>::Scalar {
        self.envelope().distance_2(point)
    }

    fn contains_point(&self, point: &<Self::Envelope as Envelope>::Point) -> bool {
        self.envelope().contains_point(point)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VerticalFeature {
    base_height: i32,
    width: Box1<i32>,
}

#[derive(Default, Debug)]
pub struct Schema {
    features: RTree<Feature>,
    //vertical_features: RTree<VerticalFeature>,
}

impl Schema {
    pub fn add(&mut self, f: Feature) {
        self.features.insert(f);
        match f {
            Feature::GroundBlock(_, _, b) => self.features.insert(Feature::FlatGround(
                Place::new(b.x.lo_incl, b.y.hi_excl - 1),
                b.x.size() as u32,
            )),
            Feature::HillBlock {
                start_x,
                height,
                lr,
                ..
            } => self.features.insert(Feature::SlopedGround {
                start: Place::new(
                    start_x,
                    match lr {
                        LR::L => height.lo_incl,
                        LR::R => height.hi_excl - 1,
                    },
                ),
                height: height.size() * -i32::from(lr),
            }),
            Feature::BigMushroomTop(p, width) => self.features.insert(Feature::FlatGround(
                p - Place::new(-(width as i32), 0),
                width,
            )),
            Feature::Igloo { .. }
            | Feature::Tile(_, _)
            | Feature::CrateCrossRect(_)
            | Feature::CrateRandomRect(_)
            | Feature::SurfaceWater(_)
            | Feature::SurfaceLava(_)
            | Feature::BigMushroomStem(_, _)
            | Feature::SlopedGround { .. }
            | Feature::FlatGround(_, _)
            | Feature::Zone(_, _)
            | Feature::Offscreen(_) => (),
        }
    }

    // pub fn add_vertical(&mut self, v: VerticalFeature) {
    //     self.vertical_features.insert(v);
    // }

    pub fn intersecting(&self, b: Box2<i32>) -> Copied<LocateInEnvelopeIntersecting<Feature>> {
        self.features
            .locate_in_envelope_intersecting(&b.into())
            .copied()
    }

    pub fn at_point(&self, p: Place) -> Copied<LocateAllAtPoint<Feature>> {
        self.features.locate_all_at_point(&p.into()).copied()
    }

    pub fn bounds(&self) -> Box2<i32> {
        let aabb = self.features.root().envelope();
        Box2::new(aabb.lower(), (aabb.upper().0 + 1, aabb.upper().1 + 1))
    }
}
