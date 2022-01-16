use bevy::math::IVec2;
use extent::Extent;
use itertools::iproduct;
use noise::NoiseFn;
use rand::{prelude::SliceRandom, Rng};
use rand_pcg::Pcg64;

use crate::assets::SHEET_W;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LR {
    L,
    R,
}

impl From<LR> for u16 {
    fn from(lr: LR) -> u16 {
        match lr {
            LR::L => 0,
            LR::R => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LMR {
    L,
    M,
    R,
}

impl From<LMR> for u16 {
    fn from(lr: LMR) -> u16 {
        match lr {
            LMR::L => 0,
            LMR::M => 1,
            LMR::R => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TMB {
    T,
    M,
    B,
}

impl From<TMB> for u16 {
    fn from(lr: TMB) -> u16 {
        match lr {
            TMB::T => 0,
            TMB::M => 1,
            TMB::B => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Slope {
    UpRight,
    DownLeft,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Terrain {
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

impl From<Terrain> for u16 {
    fn from(g: Terrain) -> u16 {
        match g {
            Terrain::PlainBlock => 1,
            Terrain::Block => 0,
            Terrain::Ground(lmr) => 15 + u16::from(lmr),
            Terrain::Concave(lr) => 5 + u16::from(lr),
            Terrain::Convex(lr) => 3 + u16::from(lr),
            Terrain::Interior => 2,
            Terrain::Slope(Slope::DownLeft) => 14,
            Terrain::Slope(Slope::UpRight) => 11,
            Terrain::SlopeInt(Slope::DownLeft) => 13,
            Terrain::SlopeInt(Slope::UpRight) => 12,
            Terrain::LedgeBlock => 7,
            Terrain::Ledge(lmr) => 8 + u16::from(lmr),
            Terrain::LedgeCap(lr) => 18 + u16::from(lr),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainTheme {
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

impl From<TerrainTheme> for u16 {
    fn from(g: TerrainTheme) -> u16 {
        match g {
            TerrainTheme::Grass => 21,
            TerrainTheme::Sand => 23,
            TerrainTheme::Snow => 24,
            TerrainTheme::Stone => 22,
            TerrainTheme::Dirt => 20,
            TerrainTheme::Castle => 19,
            TerrainTheme::Cake => 25,
            TerrainTheme::Choco => 27,
            TerrainTheme::Tundra => 31,
            TerrainTheme::Metal => 32,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IglooPiece {
    Top(LMR),
    Interior,
    InteriorAlt,
    Door,
}
impl From<IglooPiece> for u16 {
    fn from(t: IglooPiece) -> u16 {
        match t {
            IglooPiece::Top(lmr) => u16::from(lmr),
            IglooPiece::Interior => SHEET_W,
            IglooPiece::InteriorAlt => 1 + SHEET_W,
            IglooPiece::Door => 2 + SHEET_W,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreePiece {
    Top { snow: bool },
    PineBranch { snow: bool, lr: LR, double: bool },
    PineTrunk,
}

impl From<TreePiece> for u16 {
    fn from(t: TreePiece) -> u16 {
        match t {
            TreePiece::Top { snow } => SHEET_W * u16::from(snow),
            TreePiece::PineBranch { snow, lr, double } => {
                1 + u16::from(lr) + 2 * u16::from(!snow) + SHEET_W * u16::from(double)
            }
            TreePiece::PineTrunk => 5 + SHEET_W,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeTrunkPiece {
    Straight,
    Fork(LR),
    DeadFork,
    KnotBranch(LR),
    Branch(LMR),
    Base,
    BaseNarrow,
    BaseSnow,
    BaseSnowPile,
}

impl From<TreeTrunkPiece> for u16 {
    fn from(t: TreeTrunkPiece) -> u16 {
        match t {
            TreeTrunkPiece::Straight => 0,
            TreeTrunkPiece::Fork(lr) => 1 + u16::from(lr),
            TreeTrunkPiece::DeadFork => 3,
            TreeTrunkPiece::KnotBranch(lr) => 4 + 2 * u16::from(lr),
            TreeTrunkPiece::Branch(lmr) => 4 + SHEET_W + u16::from(lmr),
            TreeTrunkPiece::Base => SHEET_W,
            TreeTrunkPiece::BaseNarrow => 1 + SHEET_W,
            TreeTrunkPiece::BaseSnow => 2 + SHEET_W,
            TreeTrunkPiece::BaseSnowPile => 3 + SHEET_W,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoxPiece {
    Item { used: bool },
    Coin { used: bool },
    Crate,
}

impl From<BoxPiece> for u16 {
    fn from(b: BoxPiece) -> u16 {
        use BoxPiece::*;
        match b {
            Item { used } => 1 * SHEET_W + (1 - u16::from(used)),
            Coin { used } => 2 * SHEET_W + (1 - u16::from(used)),
            Crate => 3 * SHEET_W,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Ground(Terrain, TerrainTheme),
    Igloo(IglooPiece),
    Tree(TreePiece),
    Trunk(TreeTrunkPiece),
    LogLedge,
    Box(BoxPiece),
    Air,
}
use Tile::*;

use super::{Chunk, Gen};

impl From<Tile> for u16 {
    fn from(t: Tile) -> u16 {
        match t {
            Tile::Air => 0,
            Tile::Ground(t, s) => u16::from(t) + SHEET_W * u16::from(s),
            Tile::Igloo(piece) => 20 + SHEET_W * 14 + u16::from(piece),
            Tile::Tree(piece) => 20 + SHEET_W * 11 + u16::from(piece),
            Tile::Trunk(piece) => 16 + SHEET_W * 8 + u16::from(piece),
            Tile::LogLedge => 11 + 17 * SHEET_W,
            Tile::Box(piece) => 10 + 13 * SHEET_W + u16::from(piece),
        }
    }
}

