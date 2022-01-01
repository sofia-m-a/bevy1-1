use crate::assets::{SHEET_H, SHEET_W};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LR {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LMR {
    Left,
    Mid,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Slope {
    UpRight,
    DownLeft,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GroundTileType {
    PlainBlock,
    Block,
    Ground(LMR),
    Concave(LR),
    Convex(LR),
    Interior,
    Slope(Slope),
    SlopeInt(Slope),
    LedgeBlock,
    Ledge(LMR),
    LedgeCap(LR),
}

impl From<GroundTileType> for u16 {
    fn from(g: GroundTileType) -> u16 {
        match g {
            GroundTileType::PlainBlock => 1,
            GroundTileType::Block => 0,
            GroundTileType::Ground(LMR::Left) => 15,
            GroundTileType::Ground(LMR::Mid) => 16,
            GroundTileType::Ground(LMR::Right) => 17,
            GroundTileType::Concave(LR::Left) => 5,
            GroundTileType::Concave(LR::Right) => 6,
            GroundTileType::Convex(LR::Left) => 3,
            GroundTileType::Convex(LR::Right) => 4,
            GroundTileType::Interior => 2,
            GroundTileType::Slope(Slope::DownLeft) => 14,
            GroundTileType::Slope(Slope::UpRight) => 11,
            GroundTileType::SlopeInt(Slope::DownLeft) => 13,
            GroundTileType::SlopeInt(Slope::UpRight) => 12,
            GroundTileType::LedgeBlock => 7,
            GroundTileType::Ledge(LMR::Left) => 8,
            GroundTileType::Ledge(LMR::Mid) => 9,
            GroundTileType::Ledge(LMR::Right) => 10,
            GroundTileType::LedgeCap(LR::Left) => 18,
            GroundTileType::LedgeCap(LR::Right) => 19,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GroundSet {
    Grass,
    Sand,
    Snow,
    Stone,
    Dirt,
    Castle,
    Cake,
    Choco,
    Tundra,
    Metal,
}

impl From<GroundSet> for u16 {
    fn from(g: GroundSet) -> u16 {
        match g {
            GroundSet::Grass => 21,
            GroundSet::Sand => 23,
            GroundSet::Snow => 24,
            GroundSet::Stone => 22,
            GroundSet::Dirt => 20,
            GroundSet::Castle => 19,
            GroundSet::Cake => 25,
            GroundSet::Choco => 27,
            GroundSet::Tundra => 31,
            GroundSet::Metal => 32,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IglooPiece {
    Top(LMR),
    Interior,
    InteriorAlt,
    Door,
}

impl From<IglooPiece> for u16 {
    fn from(t: IglooPiece) -> u16 {
        match t {
            IglooPiece::Top(LMR::Left) => 0,
            IglooPiece::Top(LMR::Mid) => 1,
            IglooPiece::Top(LMR::Right) => 2,
            IglooPiece::Interior => SHEET_W,
            IglooPiece::InteriorAlt => 1 + SHEET_W,
            IglooPiece::Door => 2 + SHEET_W,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Ground(GroundTileType, GroundSet),
    Igloo(IglooPiece),
    Air,
}

impl From<Tile> for u16 {
    fn from(t: Tile) -> u16 {
        match t {
            Tile::Air => 0,
            Tile::Ground(t, s) => u16::from(t) + SHEET_W * u16::from(s),
            Tile::Igloo(piece) => 20 + SHEET_W * 14 + u16::from(piece),
        }
    }
}

// pub trait Brush {
//     type SampleSpace;
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub enum Axis {
//     Hor,
//     Ver,
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub struct XRange {
//     pub axis: Axis,
//     pub start: i32,
//     pub end: i32,
// }

// impl Brush for XRange {
//     type SampleSpace = i32;
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub struct Product<A, B>(pub A, pub B);

// impl<A: Brush, B: Brush> Brush for Product<A, B> {
//     type SampleSpace = (A::SampleSpace, B::SampleSpace);
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub struct Union<A>(pub A, pub A);

// impl<A: Brush> Brush for Union<A> {
//     type SampleSpace = A::SampleSpace;
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub struct Intersection<A>(pub A);

// impl<A: Brush> Brush for Intersection<A> {
//     type SampleSpace = A::SampleSpace;
// }
