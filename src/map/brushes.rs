use extent::Extent;
use rand::{prelude::SliceRandom, Rng};
use rand_pcg::Pcg64;

use crate::assets::SHEET_W;

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Slope {
    UpRight,
    DownLeft,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
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
#[derive(Clone, Copy, PartialEq, Eq)]
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
pub enum Tile {
    Ground(Terrain, TerrainTheme),
    Igloo(IglooPiece),
    Tree(TreePiece),
    Trunk(TreeTrunkPiece),
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
        }
    }
}

// #[derive(Clone, Copy)]
// pub struct Place {
//     pub h: Extent<u32>,
//     pub w: Extent<u32>,
//     pub x: u32,
//     pub y: u32,
// }

// pub fn range(r: &mut Pcg64, range: Extent<u32>) -> u32 {
//     let (l, h) = (range.lo().unwrap(), range.hi().unwrap());
//     r.gen_range(l..=h)
// }

// pub fn lmr(p: Place) -> Option<LMR> {
//     let (l, r) = (p.w.lo()?, p.w.hi()?);
//     Some(if p.x == l {
//         LMR::Left
//     } else if p.x == r {
//         LMR::Right
//     } else {
//         LMR::Mid
//     })
// }

// pub fn tmb(p: Place) -> Option<TMB> {
//     let (t, b) = (p.h.lo()?, p.h.hi()?);
//     Some(if p.y == t {
//         TMB::Top
//     } else if p.x == b {
//         TMB::Bottom
//     } else {
//         TMB::Mid
//     })
// }

// pub fn top(p: Place) -> Option<(Place, LMR)> {
//     let h = p.h.hi()?;
//     let lmr = lmr(p)?;
//     at_y(p, h).map(|p| (p, lmr))
// }

// pub fn bottom(p: Place) -> Option<(Place, LMR)> {
//     let l = p.h.lo()?;
//     let lmr = lmr(p)?;
//     at_y(p, l).map(|p| (p, lmr))
// }

// pub fn left(p: Place) -> Option<(Place, LMR)> {
//     let l = p.w.lo()?;
//     let lmr = lmr(p)?;
//     at_y(p, l).map(|p| (p, lmr))
// }

// pub fn right(p: Place) -> Option<(Place, LMR)> {
//     let r = p.w.hi()?;
//     let lmr = lmr(p)?;
//     at_y(p, r).map(|p| (p, lmr))
// }

// pub fn at_x(p: Place, x: u32) -> Option<Place> {
//     let d = Place {
//         w: Extent::new(x, x),
//         h: p.h,
//         x: p.x,
//         y: p.y,
//     };
//     (p.x == x).then_some(d)
// }

// pub fn at_y(p: Place, y: u32) -> Option<Place> {
//     let d = Place {
//         h: Extent::new(y, y),
//         w: p.w,
//         x: p.x,
//         y: p.y,
//     };
//     (p.y == y).then_some(d)
// }

// pub fn igloo(c: Place, r: &mut Pcg64, door: u32) -> Option<IglooPiece> {
//     top(c)
//         .map(|(_, lmr)| IglooPiece::Top(lmr))
//         .or_else(|| {
//             bottom(c)
//                 .and_then(|(p, _)| at_x(p, door))
//                 .map(|_| IglooPiece::Door)
//         })
//         .or_else(|| {
//             [IglooPiece::Interior, IglooPiece::InteriorAlt]
//                 .choose(r)
//                 .copied()
//         })
// }

// pub fn to_lr(l: LMR) -> Option<LR> {
//     match l {
//         LMR::Left => Some(LR::Left),
//         LMR::Right => Some(LR::Right),
//         LMR::Mid => None,
//     }
// }

// pub fn pine_tree_top(p: Place, r: &mut Pcg64, snowy: f64) -> Option<TreePiece> {
//     let snowy = r.gen_bool(snowy);
//     top(p)
//         .and_then(|(_, lmr)| (lmr == LMR::Mid).then_some(TreePiece::Top { snow: snowy }))
//         .or_else(|| {
//             lmr(p).and_then(|l| {
//                 to_lr(l).map(|lr| TreePiece::PineBranch {
//                     snow: snowy,
//                     lr: lr,
//                     double: r.gen_bool(0.5),
//                 })
//             })
//         })
//         .or_else(|| Some(TreePiece::PineTrunk))
// }

#[derive(Clone, Copy, Debug)]
pub struct BrushSize {
    w: Extent<u32>,
    h: Extent<u32>,
}

#[derive(Clone)]
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
                end += new.h.len();
                w = w.union(new.w);
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
                end += new.w.len();
                h = h.union(new.h);
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
                    c[(i, j)].layers[layer].tile = *tile.choose(r).unwrap();
                    dbg!(size.clone());
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

use super::map::Chunk;

// // parameters: height of leaves, height of trunk
// const pine_tree: Plan = VStack(&[
//     (
//         Some(1),
//         HStack(&[
//             (Some(1), Constant(Air)),
//             (Some(1), Constant(Tree(TreePiece::Top { snow: false }))),
//             (Some(1), Constant(Air)),
//         ]),
//     ),
//     (
//         None,
//         Choice(&[
//             HStack(&[
//                 (
//                     Some(1),
//                     Constant(Tree(TreePiece::PineBranch {
//                         snow: false,
//                         double: false,
//                         lr: LR::L,
//                     })),
//                 ),
//                 (Some(1), Constant(Tree(TreePiece::PineTrunk))),
//                 (
//                     Some(1),
//                     Constant(Tree(TreePiece::PineBranch {
//                         snow: false,
//                         double: false,
//                         lr: LR::R,
//                     })),
//                 ),
//             ]),
//             HStack(&[
//                 (
//                     Some(1),
//                     Constant(Tree(TreePiece::PineBranch {
//                         snow: false,
//                         double: true,
//                         lr: LR::L,
//                     })),
//                 ),
//                 (Some(1), Constant(Tree(TreePiece::PineTrunk))),
//                 (
//                     Some(1),
//                     Constant(Tree(TreePiece::PineBranch {
//                         snow: false,
//                         double: true,
//                         lr: LR::R,
//                     })),
//                 ),
//             ]),
//         ]),
//     ),
//     (None, todo!()),
// ]);
