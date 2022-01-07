use extent::Extent;
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

#[derive(Clone, Copy, Debug)]
pub struct BrushSize {
    w: Extent<u32>,
    h: Extent<u32>,
}

#[derive(Clone, Debug)]
pub enum Plan {
    VStack(Vec<Plan>),
    HStack(Vec<Plan>),
    Choice(Vec<Plan>),
    Constant((u32, u32), Vec<Tile>),
}
use Plan::*;

pub fn run_plan(
    p: Plan,
    c: &mut Chunk,
    r: &mut Pcg64,
    center: (u32, u32),
    layer: usize,
) -> BrushSize {
    match p {
        VStack(vs) => {
            let mut end = center.1;
            let mut w = Extent::empty();
            for plan in vs {
                let center = (center.0, end);
                let new = run_plan(plan, c, r, center, layer);
                dbg!(w, new.w, w.union(new.w));
                end += new.h.len();
                w = w.union(new.w);
                //println!("V: {:?} {:?} {:?} {:?}", end, w, center, new);
            }

            BrushSize {
                h: Extent::new(center.1, end - 1),
                w,
            }
        }
        HStack(hs) => {
            let mut end = center.0;
            let mut h = Extent::empty();
            for plan in hs {
                let center = (end, center.1);
                let new = run_plan(plan, c, r, center, layer);
                dbg!(h, new.h, h.union(new.h));
                end += new.w.len();
                h = h.union(new.h);
                //println!("H: {:?} {:?} {:?} {:?}", end, h, center, new);
            }

            BrushSize {
                w: Extent::new(center.1, end - 1),
                h,
            }
        }
        Choice(options) => {
            let option = options.choose(r);
            if let Some(o) = option {
                run_plan(o.clone(), c, r, center, layer)
            } else {
                BrushSize {
                    w: Extent::empty(),
                    h: Extent::empty(),
                }
            }
        }
        Constant(size, tile) => {
            let size = BrushSize {
                w: (center.0..center.0 + size.0).into(),
                h: (center.1..center.1 + size.1).into(),
            };
            for i in size.w.iter() {
                for j in size.h.iter() {
                    c[(i, j)].tile = *tile.choose(r).unwrap();
                }
            }
            size
        }
    }
}

pub fn igloo(size: (u32, u32), r: &mut Pcg64) -> Plan {
    let door = r.gen_range(0..size.0);

    Plan::VStack(vec![
        Plan::HStack(vec![
            Plan::Constant(
                (door, 1),
                vec![Igloo(IglooPiece::Interior), Igloo(IglooPiece::InteriorAlt)],
            ),
            Plan::Constant((1, 1), vec![Igloo(IglooPiece::Door)]),
            Plan::Constant(
                (size.0 - door - 1, 1),
                vec![Igloo(IglooPiece::Interior), Igloo(IglooPiece::InteriorAlt)],
            ),
        ]),
        Plan::Constant(
            (size.0, size.0 - 2),
            vec![Igloo(IglooPiece::Interior), Igloo(IglooPiece::InteriorAlt)],
        ),
        Plan::HStack(vec![
            Plan::Constant((1, 1), vec![Igloo(IglooPiece::Top(LMR::L))]),
            Plan::Constant((size.0 - 2, 1), vec![Igloo(IglooPiece::Top(LMR::M))]),
            Plan::Constant((1, 1), vec![Igloo(IglooPiece::Top(LMR::R))]),
        ]),
    ])
}

use super::Chunk;

pub fn pine_tree(snow: bool, trunk_height: u32, leaf_height: u32) -> Plan {
    VStack(vec![
        HStack(vec![
            Constant((1, 1), vec![Tile::Air]),
            Constant(
                (1, 1),
                if snow {
                    vec![
                        Tile::Trunk(TreeTrunkPiece::Base),
                        Tile::Trunk(TreeTrunkPiece::BaseNarrow),
                        Tile::Trunk(TreeTrunkPiece::BaseSnow),
                        Tile::Trunk(TreeTrunkPiece::BaseSnowPile),
                    ]
                } else {
                    vec![
                        Tile::Trunk(TreeTrunkPiece::Base),
                        Tile::Trunk(TreeTrunkPiece::BaseNarrow),
                    ]
                },
            ),
        ]),
        Choice(vec![
            HStack(vec![
                Constant(
                    (1, 1),
                    vec![Tile::Tree(TreePiece::PineBranch {
                        snow,
                        double: false,
                        lr: LR::L,
                    })],
                ),
                Constant((1, 1), vec![Tile::Tree(TreePiece::PineTrunk)]),
                Constant(
                    (1, 1),
                    vec![Tile::Tree(TreePiece::PineBranch {
                        snow,
                        double: false,
                        lr: LR::R,
                    })],
                ),
            ]),
            HStack(vec![
                Constant(
                    (1, 1),
                    vec![Tile::Tree(TreePiece::PineBranch {
                        snow,
                        double: true,
                        lr: LR::L,
                    })],
                ),
                Constant((1, 1), vec![Tile::Tree(TreePiece::PineTrunk)]),
                Constant(
                    (1, 1),
                    vec![Tile::Tree(TreePiece::PineBranch {
                        snow,
                        double: true,
                        lr: LR::R,
                    })],
                ),
            ]),
        ]),
        HStack(vec![
            Constant((1, 1), vec![Tile::Air]),
            Constant((1, 1), vec![Tile::Tree(TreePiece::Top { snow })]),
            Constant((1, 1), vec![Tile::Air]),
        ]),
    ])
}
