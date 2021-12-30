use crate::assets::{SHEET_H, SHEET_W};

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GroundTileType {
    PlainBlock,
    Block,
    Left,
    Mid,
    Right,
    LeftCave,
    RightCave,
    LeftVex,
    RightVex,
    Interior,
    SlopeDown,
    SlopeUp,
    SlopeDownInt,
    SlopeUpInt,
    LedgeBlock,
    LedgeLeft,
    LedgeMid,
    LedgeRight,
    LedgeCapLeft,
    LedgeCapRigtht,
}

impl From<GroundTileType> for u16 {
    fn from(g: GroundTileType) -> u16 {
        match g {
            GroundTileType::PlainBlock => 1,
            GroundTileType::Block => 0,
            GroundTileType::Left => 15,
            GroundTileType::Mid => 16,
            GroundTileType::Right => 17,
            GroundTileType::LeftCave => 5,
            GroundTileType::RightCave => 6,
            GroundTileType::LeftVex => 3,
            GroundTileType::RightVex => 4,
            GroundTileType::Interior => 2,
            GroundTileType::SlopeDown => 14,
            GroundTileType::SlopeUp => 11,
            GroundTileType::SlopeDownInt => 13,
            GroundTileType::SlopeUpInt => 12,
            GroundTileType::LedgeBlock => 7,
            GroundTileType::LedgeLeft => 8,
            GroundTileType::LedgeMid => 9,
            GroundTileType::LedgeRight => 10,
            GroundTileType::LedgeCapLeft => 18,
            GroundTileType::LedgeCapRigtht => 19,
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
pub enum Tile {
    Ground(GroundTileType, GroundSet),
    Air,
}

impl From<Tile> for u16 {
    fn from(t: Tile) -> u16 {
        match t {
            Tile::Air => 0,
            Tile::Ground(t, s) => u16::from(t) + SHEET_W * u16::from(s),
        }
    }
}
