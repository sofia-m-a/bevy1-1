use crate::assets::SHEET_W;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LR {
    Left,
    Right,
}

impl From<LR> for u16 {
    fn from(lr: LR) -> u16 {
        match lr {
            LR::Left => 0,
            LR::Right => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LMR {
    Left,
    Mid,
    Right,
}

impl From<LMR> for u16 {
    fn from(lr: LMR) -> u16 {
        match lr {
            LMR::Left => 0,
            LMR::Mid => 1,
            LMR::Right => 2,
        }
    }
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
            GroundTileType::Ground(lmr) => 15 + u16::from(lmr),
            GroundTileType::Concave(lr) => 5 + u16::from(lr),
            GroundTileType::Convex(lr) => 3 + u16::from(lr),
            GroundTileType::Interior => 2,
            GroundTileType::Slope(Slope::DownLeft) => 14,
            GroundTileType::Slope(Slope::UpRight) => 11,
            GroundTileType::SlopeInt(Slope::DownLeft) => 13,
            GroundTileType::SlopeInt(Slope::UpRight) => 12,
            GroundTileType::LedgeBlock => 7,
            GroundTileType::Ledge(lmr) => 8 + u16::from(lmr),
            GroundTileType::LedgeCap(lr) => 18 + u16::from(lr),
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
            IglooPiece::Top(lmr) => u16::from(lmr),
            IglooPiece::Interior => SHEET_W,
            IglooPiece::InteriorAlt => 1 + SHEET_W,
            IglooPiece::Door => 2 + SHEET_W,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
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
                u16::from(lr) + 2 * u16::from(snow) + SHEET_W * u16::from(double)
            }
            TreePiece::PineTrunk => 5 + SHEET_W,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
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
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Ground(GroundTileType, GroundSet),
    Igloo(IglooPiece),
    Tree(TreePiece),
    Trunk(TreeTrunkPiece),
    Air,
}

impl From<TileType> for u16 {
    fn from(t: TileType) -> u16 {
        match t {
            TileType::Air => 0,
            TileType::Ground(t, s) => u16::from(t) + SHEET_W * u16::from(s),
            TileType::Igloo(piece) => 20 + SHEET_W * 14 + u16::from(piece),
            TileType::Tree(piece) => 20 + SHEET_W * 11 + u16::from(piece),
            TileType::Trunk(piece) => 16 + SHEET_W * 8 + u16::from(piece),
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
