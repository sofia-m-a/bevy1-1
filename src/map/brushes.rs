use std::collections::HashSet;

use bevy::prelude::Resource;
use bevy::prelude::UVec2;
use enum_iterator::Sequence;
use extent::Extent;
use itertools::iproduct;
use itertools::Itertools;
use itertools::Position::*;
use ndarray::s;
use noise::NoiseFn;
use noise::OpenSimplex;
use noise::Seedable;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use num_traits::PrimInt;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use rstar::*;

use crate::map::CHUNK_SIZE;

use super::Place;

type Index = (u16, u16);
type Alt = bool;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Alt3 {
    Alt0,
    Alt1,
    Alt2,
}

impl From<Alt3> for u16 {
    fn from(a: Alt3) -> Self {
        match a {
            Alt3::Alt0 => 0,
            Alt3::Alt1 => 1,
            Alt3::Alt2 => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Alt5 {
    Alt0,
    Alt1,
    Alt2,
    Alt3,
    Alt4,
}

impl From<Alt5> for u16 {
    fn from(a: Alt5) -> Self {
        match a {
            Alt5::Alt0 => 0,
            Alt5::Alt1 => 1,
            Alt5::Alt2 => 2,
            Alt5::Alt3 => 3,
            Alt5::Alt4 => 4,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Sequence, FromPrimitive)]
pub enum Terrain {
    Cake,
    Choco,
    Metal,
    Tundra,
    Castle,
    Dirt,
    Grass,
    Stone,
    Sand,
    Snow,
    Industrial,
}

impl From<Terrain> for Index {
    fn from(t: Terrain) -> Self {
        (0, t as u16 * 4)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LR {
    L,
    R,
}

impl From<LR> for u16 {
    fn from(lr: LR) -> Self {
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
    fn from(lmr: LMR) -> Self {
        match lmr {
            LMR::L => 0,
            LMR::M => 1,
            LMR::R => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TB {
    T,
    B,
}

impl From<TB> for u16 {
    fn from(tb: TB) -> Self {
        match tb {
            TB::T => 0,
            TB::B => 1,
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
    fn from(tmb: TMB) -> Self {
        match tmb {
            TMB::T => 0,
            TMB::M => 1,
            TMB::B => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LRTB {
    L,
    R,
    T,
    B,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainTile {
    RoundLedge(LR),
    OverLedge(LR),
    SlopeLedge(LR),
    BlockLedge(LR),
    BlockFace(LMR, TMB),
    Slope(LR),
    SlopeInt(LR),
    Cap(LR),
    FaceInt(LR, TB),
    RockSlope(LR, TB),
    Block,
    BareBlock,
    Single,
    SingleBare,
    SingleHalf(Alt),
    Half(Alt, LMR),
    Jagged,
}

impl From<TerrainTile> for Index {
    fn from(t: TerrainTile) -> Self {
        match t {
            TerrainTile::RoundLedge(lr) => (0 + u16::from(lr), 0),
            TerrainTile::OverLedge(lr) => (2 + u16::from(lr), 0),
            TerrainTile::SlopeLedge(lr) => (4 + u16::from(lr), 0),
            TerrainTile::BlockLedge(lr) => (6 + u16::from(lr), 0),
            TerrainTile::BlockFace(lmr, tmb) => (u16::from(lmr), 1 + u16::from(tmb)),
            TerrainTile::Slope(lr) => (3 + u16::from(lr), 1),
            TerrainTile::SlopeInt(lr) => (3 + u16::from(lr), 2),
            TerrainTile::Cap(lr) => (6 + u16::from(lr), 3),
            TerrainTile::FaceInt(lr, tb) => (5 + u16::from(lr), 2 - u16::from(tb)),
            TerrainTile::RockSlope(lr, tb) => (9 + u16::from(lr), 2 + u16::from(tb)),
            TerrainTile::Block => (8, 2),
            TerrainTile::BareBlock => (3, 3),
            TerrainTile::Single => (4, 3),
            TerrainTile::SingleBare => (5, 3),
            TerrainTile::SingleHalf(a) => (8, u16::from(a)),
            TerrainTile::Half(a, lmr) => (9 + u16::from(lmr), u16::from(a)),
            TerrainTile::Jagged => (8, 3),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RockType {
    Sandstone,
    Slate,
    Stone,
}

impl From<RockType> for Index {
    fn from(rt: RockType) -> Self {
        match rt {
            RockType::Sandstone => (12, 0),
            RockType::Slate => (12, 3),
            RockType::Stone => (12, 6),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Roof {
    Brick,
    Slate,
    Straw,
}

impl From<Roof> for Index {
    fn from(r: Roof) -> Self {
        match r {
            Roof::Brick => (16, 0),
            Roof::Slate => (16, 3),
            Roof::Straw => (16, 6),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CaveTile {
    Slope(LR, TB),
    Spike(TB),
    Jagged(TB),
    BigRock,
    SmallRock,
}

impl From<CaveTile> for Index {
    fn from(c: CaveTile) -> Self {
        match c {
            CaveTile::Slope(lr, tb) => (19 + u16::from(lr), u16::from(tb)),
            CaveTile::Spike(tb) => (21, u16::from(tb)),
            CaveTile::Jagged(tb) => (22, u16::from(tb)),
            CaveTile::BigRock => (23, 0),
            CaveTile::SmallRock => (23, 1),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cave {
    Dirt,
    Stone,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color4 {
    Y,
    R,
    G,
    B,
}

impl From<Color4> for u16 {
    fn from(c: Color4) -> Self {
        match c {
            Color4::Y => 0,
            Color4::R => 1,
            Color4::G => 2,
            Color4::B => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeTile {
    Top { snow: bool },
    PineBranch { snow: bool, lr: LR, double: bool },
    PineTrunk,
}

impl From<TreeTile> for Index {
    fn from(tt: TreeTile) -> Self {
        match tt {
            TreeTile::Top { snow } => (17, 16 + u16::from(snow)),
            TreeTile::PineBranch { snow, lr, double } => (
                20 + u16::from(lr) - 2 * u16::from(snow),
                16 + u16::from(double),
            ),
            TreeTile::PineTrunk => (22, 17),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Air,

    Terrain(Terrain, TerrainTile),
    MetalTri,
    MetalYellowSquare,

    Building(RockType, LMR, TMB),
    BuildingInt(RockType, Alt),
    Roof(Roof, LMR, TB),

    Cave(Cave, CaveTile),

    BigSnowball,
    SmallSnowball,
    GroundSnowball,
    SnowPile,

    GreenArrow(LRTB),

    MetalFence(LMR),
    MetalUpper,
    MetalUpperWire {
        long: bool,
    },
    GirderSmall {
        bolts: bool,
    },
    Girder {
        bolts: bool,
    },
    GirderHoles {
        bolts: bool,
    },
    Railing(Alt),
    Beam(LR),
    Strut(LR, TB),
    Hook(TB),

    Gummy(Color4),
    Cherry,
    Heart,
    Cone,
    CookieBlackWhite,
    CookieBeigeBrown,
    CookieBeigePink,
    IcecreamBeige,
    IcecreamWhite,
    IcecreamPink,
    IcecreamBrown,
    WaferWhite,
    WaferPink,
    WaferBrown,
    GummyWormHookGreenYellow,
    GummyWormLoopGreenYellow,
    GummyWormTailGreenYellow,
    GummyWormHookRedWhite,
    GummyWormLoopRedWhite,
    GummyWormTailRedWhite,
    CandyPoleBrown(TB),
    CandyPoleGreen(TB),
    CandyPolePink(TB),
    CandyPoleRed(TB),
    LittleCandyCaneRed,
    CandyCaneBaseRed,
    CandyCaneTopRed(Alt),
    LittleCandyCaneGreen,
    CandyCaneBaseGreen,
    CandyCaneTopGreen(Alt),
    LittleCandyCanePink,
    CandyCaneBasePink,
    CandyCaneTopPink(Alt),
    LollipopStickWhite,
    LollipopStickBeige,
    LollipopStickBrown,
    LollipopBaseBeige,
    LollipopBasePink,
    LollipopGreen,
    LollipopRed,
    LollipopYellow,
    LollipopGreenSwirl(Alt),
    LollipopRedSwirl(Alt),

    MetalBoxWire(Alt),
    MetalBoxCross(Alt),
    MetalBoxSlash(Alt),
    MetalBoxBlank(Alt),

    SlimeSingle(TB),
    Slime(LMR, TB),
    SlimeBubble(TB),

    PineTree(TreeTile),
    TrunkStraight,
    TrunkFork(LR),
    TrunkDeadFork,
    TrunkKnotBranch(LR),
    TrunkBranch(LMR),
    TrunkBase,
    TrunkBaseNarrow,
    TrunkBaseSnow,
    TrunkBaseSnowPile,

    LavaWave,
    LavaSingle,
    Lava,
    WaterWave,
    WaterSingle,
    Water,
    IceWaterWave,
    IceWaterSingle,
    IceWater,
    SparklingWaterWave,
    SparklingWaterSingle,
    SparklingWater,
    DeepWater(Alt),
    IceBlock,
    SparkleIceBlock,
    IceHalf,
    SparkleIceHalf,

    MushroomBlockCaramel(Alt, LMR),
    MushroomStemBlockCaramel(Alt),
    MushroomBlockBrown(Alt, LMR),
    MushroomStemBlockBrown(Alt),
    MushroomBlockRed(Alt, LMR),
    MushroomStemBlockRed(Alt),
    MushroomBlockWhite(Alt, LMR),
    MushroomStemBlockWhite(Alt),
    MushroomStemTop(Alt),
    MushroomStemLeaf,
    MushroomStemRing(Alt),
    MushroomStem,
    MushroomStemBase(Alt),
    MushroomWhite(Alt),
    MushroomRed(Alt),
    MushroomBrown(Alt),

    BrickBlock,
    StoneBlock,
    CrateBlank,
    CrateSlash,
    CrateCross,
    CrateSquareBang,
    CrateTriangleBang,
    BangBox {
        empty: bool,
        alt: Alt,
    },
    CoinBox {
        empty: bool,
        alt: Alt,
    },
    TriangleBangBoxAlt {
        empty: bool,
    },

    CrenellationOverhang(LR, Alt),
    Crenellation(Alt),
    CrenellationBroken(Alt),
    CrenellationHalf,
    CrenellationHalfBroken,
    CrenellationHalfOpen,

    SnowSlope(LR),
    SnowPileBig,
    SnowPileSmall,
    SnowPileLow(LMR),
    SnowDrift,
    Icicle(TB, Alt3),

    Gate(LMR, TB, Alt),
    GateDoor(LMR, TB, Alt),
    GateSpikes(LMR, TB),
    GateBars(LMR, TB),

    GrassTuft,
    Cactus,
    Sapling,
    SaplingTall,
    IceTuft,
    IceCrystal,
    PurpleCrystal,
    Moss,
    IceMoss,
    Rock,
    IceRock(Alt),
    MossRock(Alt),
    FrozenShrub,
    StoneSpike,
    StoneSpike2(TB),
    IglooTop(LMR),
    IglooInterior(Alt),
    IglooDoor,

    Laser,
    Laser2,
    Fireball,
    Brick(Alt),
    StoneBrick(Alt),
    Star,

    TranslucentWindow(Color4),
    TranslucentWindowBox(Color4),
    BarrelSide(Color4),
    Barrel(Color4),
    LightStick(Color4),
    Light(Color4),
    Key(Color4),
    Crystal(Color4),
    FlagFrame1(Color4),
    FlagFrame2(Color4),
    FallenFlag(Color4),
    Button {
        pressed: bool,
        color: Color4,
    },
    Lock(Color4),
    LaserLever(LR, Color4),
    LaserSpark(Color4),
    LaserBeamH(Color4),
    LaserBeamV(Color4),
    TranslucentBarrel(Color4),
    Zapper(LRTB, Alt),
    Lever(LMR),
    Chain,
    Weight,
    WeightChain,
    Spring,
    SpringUp,
    Grinder(Alt),
    HalfGrinder(Alt),

    CoinBronze,
    CoinSilver,
    CoinGold,
    SwordBronze,
    SwordSilver,
    SwordGold,
    ShieldBronze,
    ShieldSilver,
    ShieldGold,
    PurpleGun,
    PurpleGunFiring,
    SilverGun,
    SilverGunFiring,
    Bomb,
    BombFlash,

    Chimney(Alt3),
    SignBed {
        hanging: bool,
    },
    SignCoin {
        hanging: bool,
    },
    SignMug {
        hanging: bool,
    },
    TorchHolder,
    Torch(LR),
    Umbrella {
        open: bool,
    },
    Clock,
    WeatherVane,
    Shade(Alt3),
    RopeVHook,
    RopeH,
    RopeV,
    Fence(Alt),
    FenceBroken,
    FenceLow,
    FenceOpen,
    FenceLowerHalf,
    FenceLower,
    Bridge,
    BridgeLog,
    SignExit,
    SignArrow(LR),
    SignBlank,

    DoorwayGrey(TB),
    DoorwayBeige(TB),
    DoorLockedGrey(TB),
    DoorLockedBeige(TB),
    DoorInsetBeige(TB),
    DoorTopWindow,
    DoorBeige,
    WindowOpenHalf,
    WindowStainedHalf,
    WindowOpen,
    Window(Alt3),
    HighWindowOpen(TMB),
    HighWindow(TMB, Alt3),

    CastleWindowOpenSmall(Alt),
    CastleWindowShutSmall(Alt),
    CastleWindowOpen(Alt),
    CastleWindowShut(Alt),
    CastleWindowOpenHigh(Alt, TMB),
    CastleWindowShutHigh(Alt, TMB),
    CastleWindowOpenSlit(Alt, TMB),
    CastleWindowShutSlit(Alt, TMB),
    CastleWindowOpenHighAlt(Alt),
    CastleWindowShutHighAlt(Alt),
    CastleWindowOpenSlitAlt(Alt),
    CastleWindowShutSlitAlt(Alt),

    BannerSmallRed(TMB, Alt),
    BannerSmallGreen(TMB, Alt),
    TapestryMidRed(Alt5),
    TapestryMidGreen(Alt5),
    TapestryBottomRed(Alt),
    TapestryBottomGreen(Alt),
    BannerRed(Alt5),
    BannerGreen(Alt5),

    FlagWoodRedFrame1(LR),
    FlagWoodRedFrame2(LR),
    FlagWoodGreenFrame1(LR),
    FlagWoodGreenFrame2(LR),
    FlagWoodLongRedFrame1(LR),
    FlagWoodLongRedFrame2(LR),
    FlagWoodLongGreenFrame1(LR),
    FlagWoodLongGreenFrame2(LR),
    FlagWoodTipRedFrame1(LR),
    FlagWoodTipRedFrame2(LR),
    FlagWoodTipGreenFrame1(LR),
    FlagWoodTipGreenFrame2(LR),
    FlagpoleTop(LMR),
    Flagpole(LMR),
    FlagpoleAlt(Alt),
    FallenFlagRed(LR),
    FallenFlagGreen(LR),
    FlagBase(Alt),
    TorchWood(LR),

    Console {
        lmr: Option<LMR>,
        on: bool,
        knobs: bool,
    },
    ConsoleButtons(LMR),
    ConsoleButtonsSmall(LMR),
    Shelves(TB),
    ShelvesNarrow(TB),
    Pillar(TMB),
    BangSticker,
    DiamondSticker,
    CoinSticker,
    TapeYellow(Alt5),
    TapeRed(Alt5),

    CrenellationsBrickTop(LMR),
    CrenellationsBrick(LMR),

    TapestryHolder(LR),
    TapestryTopRed,
    TapestryTopGreen,

    CastleStoneWall(Alt3),
    Castle(Alt, LMR, TMB),
    CastleBeamSlash(Alt, LMR),
    CastleBeamCross(Alt, LMR),
    CastleBeamSlats(Alt, TMB),
    CastleSlash(Alt, TMB),
    CastleBeamSlashV(Alt, TB),
    CastleSlat(Alt),

    CastleRoof(Alt, Alt),
    CastleRoofPeak(Alt, LR),
    CastleRoofSlope(Alt, LR),
    CastleRoofBend(Alt, LR),
    CastleRoofPeakBase(Alt, LR),
    CastleRoofLow(Alt, LR),
    CastleRoofLowCont(Alt, LR),
}

impl From<Tile> for Index {
    fn from(t: Tile) -> Self {
        match t {
            Tile::Air => (12, 32),

            Tile::Terrain(t, tt) => {
                let (x1, y1) = Index::from(t);
                let (x2, y2) = Index::from(tt);
                (x1 + x2, y1 + y2)
            }
            Tile::MetalTri => (7, 9),
            Tile::MetalYellowSquare => (7, 10),
            Tile::Building(rt, lmb, tmb) => (
                Index::from(rt).0 + u16::from(lmb),
                Index::from(rt).1 + u16::from(tmb),
            ),
            Tile::BuildingInt(rt, a) => (Index::from(rt).0 + 3, Index::from(rt).1 + u16::from(a)),
            Tile::Roof(r, lmr, tb) => (
                Index::from(r).0 + u16::from(lmr),
                Index::from(r).1 + u16::from(tb),
            ),
            Tile::Cave(c, ct) => (
                Index::from(ct).0,
                Index::from(ct).1
                    + match c {
                        Cave::Dirt => 0,
                        Cave::Stone => 2,
                    },
            ),
            Tile::BigSnowball => (23, 4),
            Tile::SmallSnowball => (22, 5),
            Tile::GroundSnowball => (21, 5),
            Tile::SnowPile => (22, 4),
            Tile::GreenArrow(lrtb) => match lrtb {
                LRTB::L => (22, 6),
                LRTB::R => (24, 6),
                LRTB::T => (23, 5),
                LRTB::B => (23, 7),
            },
            Tile::MetalFence(lmr) => (24 + u16::from(lmr), 0),
            Tile::MetalUpper => (24, 1),
            Tile::MetalUpperWire { long } => (25 + u16::from(long), 1),
            Tile::GirderSmall { bolts } => (24 + u16::from(bolts), 2),
            Tile::Girder { bolts } => (26 + u16::from(bolts), 3),
            Tile::GirderHoles { bolts } => (24 + u16::from(bolts), 2),
            Tile::Railing(a) => (26 + u16::from(a), 2),
            Tile::Beam(lr) => (24 + u16::from(lr), 4),
            Tile::Strut(lr, tb) => (26 + u16::from(lr), 4 + u16::from(tb)),
            Tile::Hook(tb) => (24 + u16::from(tb), 5),
            Tile::Gummy(c) => (12, 12 - u16::from(c)),
            Tile::Cherry => (13, 9),
            Tile::Heart => (17, 9),
            Tile::Cone => (17, 10),
            Tile::CookieBlackWhite => (13, 10),
            Tile::CookieBeigeBrown => (14, 10),
            Tile::CookieBeigePink => (14, 9),
            Tile::IcecreamBeige => (15, 9),
            Tile::IcecreamWhite => (15, 10),
            Tile::IcecreamPink => (16, 9),
            Tile::IcecreamBrown => (16, 10),
            Tile::WaferWhite => (22, 10),
            Tile::WaferPink => (23, 9),
            Tile::WaferBrown => (23, 10),
            Tile::GummyWormHookGreenYellow => (18, 9),
            Tile::GummyWormLoopGreenYellow => (19, 9),
            Tile::GummyWormTailGreenYellow => (18, 10),
            Tile::GummyWormHookRedWhite => (20, 9),
            Tile::GummyWormLoopRedWhite => (21, 9),
            Tile::GummyWormTailRedWhite => (20, 10),
            Tile::CandyPoleBrown(tb) => (20, 11 + u16::from(tb)),
            Tile::CandyPoleGreen(tb) => (21, 11 + u16::from(tb)),
            Tile::CandyPolePink(tb) => (22, 11 + u16::from(tb)),
            Tile::CandyPoleRed(tb) => (23, 11 + u16::from(tb)),
            Tile::LittleCandyCaneRed => (14, 12),
            Tile::CandyCaneBaseRed => (13, 12),
            Tile::CandyCaneTopRed(a) => (13 + u16::from(a), 11),
            Tile::LittleCandyCaneGreen => (16, 12),
            Tile::CandyCaneBaseGreen => (15, 12),
            Tile::CandyCaneTopGreen(a) => (15 + u16::from(a), 11),
            Tile::LittleCandyCanePink => (18, 12),
            Tile::CandyCaneBasePink => (17, 12),
            Tile::CandyCaneTopPink(a) => (17 + u16::from(a), 11),
            Tile::LollipopStickWhite => (12, 13),
            Tile::LollipopStickBeige => (13, 13),
            Tile::LollipopStickBrown => (14, 13),
            Tile::LollipopBaseBeige => (15, 13),
            Tile::LollipopBasePink => (16, 13),
            Tile::LollipopGreen => (17, 13),
            Tile::LollipopRed => (18, 13),
            Tile::LollipopYellow => (19, 13),
            Tile::LollipopGreenSwirl(a) => (20 + u16::from(a), 13),
            Tile::LollipopRedSwirl(a) => (22 + u16::from(a), 13),
            Tile::MetalBoxWire(a) => (24 + u16::from(a), 9),
            Tile::MetalBoxCross(a) => (24 + u16::from(a), 10),
            Tile::MetalBoxSlash(a) => (24 + u16::from(a), 11),
            Tile::MetalBoxBlank(a) => (24 + u16::from(a), 12),
            Tile::SlimeSingle(tb) => (12, 14 + u16::from(tb)),
            Tile::Slime(lmr, tb) => (13 + u16::from(lmr), 14 + u16::from(tb)),
            Tile::SlimeBubble(tb) => (16, 14 + u16::from(tb)),
            Tile::PineTree(tt) => Index::from(tt),
            Tile::TrunkStraight => (17, 14),
            Tile::TrunkFork(lr) => (18 + u16::from(lr), 14),
            Tile::TrunkDeadFork => (20, 14),
            Tile::TrunkKnotBranch(lr) => (21 + 2 * u16::from(lr), 14),
            Tile::TrunkBranch(lmr) => (21 + u16::from(lmr), 15),
            Tile::TrunkBase => (17, 15),
            Tile::TrunkBaseNarrow => (18, 15),
            Tile::TrunkBaseSnow => (19, 15),
            Tile::TrunkBaseSnowPile => (20, 15),
            Tile::LavaWave => (12, 16),
            Tile::LavaSingle => (13, 16),
            Tile::Lava => (14, 16),
            Tile::WaterWave => (12, 17),
            Tile::WaterSingle => (13, 17),
            Tile::Water => (14, 17),
            Tile::IceWaterWave => (12, 18),
            Tile::IceWaterSingle => (13, 18),
            Tile::IceWater => (14, 18),
            Tile::SparklingWaterWave => (12, 19),
            Tile::SparklingWaterSingle => (13, 19),
            Tile::SparklingWater => (14, 19),
            Tile::DeepWater(a) => (15, 18 + u16::from(a)),
            Tile::IceBlock => (15, 16),
            Tile::SparkleIceBlock => (15, 17),
            Tile::IceHalf => (16, 16),
            Tile::SparkleIceHalf => (16, 17),
            Tile::MushroomBlockCaramel(a, lmr) => (12 + u16::from(lmr), 20 + u16::from(a)),
            Tile::MushroomStemBlockCaramel(a) => (15, 20 + u16::from(a)),
            Tile::MushroomBlockBrown(a, lmr) => (12 + u16::from(lmr), 22 + u16::from(a)),
            Tile::MushroomStemBlockBrown(a) => (15, 22 + u16::from(a)),
            Tile::MushroomBlockRed(a, lmr) => (12 + u16::from(lmr), 24 + u16::from(a)),
            Tile::MushroomStemBlockRed(a) => (15, 24 + u16::from(a)),
            Tile::MushroomBlockWhite(a, lmr) => (12 + u16::from(lmr), 26 + u16::from(a)),
            Tile::MushroomStemBlockWhite(a) => (15, 26 + u16::from(a)),
            Tile::MushroomStemTop(a) => (16, 20 + u16::from(a)),
            Tile::MushroomStemLeaf => (16, 22),
            Tile::MushroomStemRing(a) => (16, 23 + u16::from(a)),
            Tile::MushroomStem => (16, 25),
            Tile::MushroomStemBase(a) => (16, 26 + u16::from(a)),
            Tile::MushroomWhite(a) => (17 + u16::from(a), 25),
            Tile::MushroomRed(a) => (17 + u16::from(a), 26),
            Tile::MushroomBrown(a) => (17 + u16::from(a), 27),
            Tile::BrickBlock => (22, 22),
            Tile::StoneBlock => (23, 22),
            Tile::CrateBlank => (19, 22),
            Tile::CrateSlash => (20, 22),
            Tile::CrateCross => (21, 22),
            Tile::CrateSquareBang => (19, 19),
            Tile::CrateTriangleBang => (20, 19),
            Tile::BangBox { empty, alt } => (19 + u16::from(empty) + 2 * u16::from(alt), 20),
            Tile::CoinBox { empty, alt } => (19 + u16::from(empty) + 2 * u16::from(alt), 21),
            Tile::TriangleBangBoxAlt { empty } => (21 + u16::from(empty), 19),
            Tile::CrenellationOverhang(lr, a) => (19 + u16::from(lr), 24 + u16::from(a)),
            Tile::Crenellation(a) => (21, 24 + u16::from(a)),
            Tile::CrenellationBroken(a) => (22, 24 + u16::from(a)),
            Tile::CrenellationHalf => (23, 24),
            Tile::CrenellationHalfBroken => (24, 24),
            Tile::CrenellationHalfOpen => (25, 24),
            Tile::SnowSlope(lr) => (12 + u16::from(lr), 28),
            Tile::SnowPileBig => (14, 28),
            Tile::SnowPileSmall => (15, 28),
            Tile::SnowPileLow(lmr) => (16 + u16::from(lmr), 28),
            Tile::SnowDrift => (19, 28),
            Tile::Icicle(tb, alt3) => (12 + u16::from(alt3), 29 + u16::from(tb)),
            Tile::Gate(lmr, tb, a) => (20 + u16::from(lmr) + 3 * u16::from(a), 27 + u16::from(tb)),
            Tile::GateDoor(lmr, tb, a) => {
                (20 + u16::from(lmr) + 3 * u16::from(a), 29 + u16::from(tb))
            }
            Tile::GateSpikes(lmr, tb) => (20 + u16::from(lmr), 31 + u16::from(tb)),
            Tile::GateBars(lmr, tb) => (23 + u16::from(lmr), 31 + u16::from(tb)),
            Tile::GrassTuft => (14, 32),
            Tile::Cactus => (15, 32),
            Tile::Sapling => (16, 32),
            Tile::SaplingTall => (17, 32),
            Tile::IceTuft => (18, 32),
            Tile::IceCrystal => (19, 32),
            Tile::PurpleCrystal => (14, 33),
            Tile::Moss => (15, 33),
            Tile::IceMoss => (16, 33),
            Tile::Rock => (12, 33),
            Tile::IceRock(a) => (12 + u16::from(a), 34),
            Tile::MossRock(a) => (12 + u16::from(a), 35),
            Tile::FrozenShrub => (14, 35),
            Tile::StoneSpike => (15, 35),
            Tile::StoneSpike2(tb) => (16, 34 + u16::from(tb)),
            Tile::IglooTop(lmr) => (17 + u16::from(lmr), 34),
            Tile::IglooInterior(a) => (17 + u16::from(a), 35),
            Tile::IglooDoor => (19, 35),
            Tile::Laser => (22, 33),
            Tile::Laser2 => (22, 34),
            Tile::Fireball => (22, 35),
            Tile::Brick(a) => (23, 33 + u16::from(a)),
            Tile::StoneBrick(a) => (24, 33 + u16::from(a)),
            Tile::Star => (23, 35),
            Tile::TranslucentWindow(c) => (0, 44 + u16::from(c)),
            Tile::TranslucentWindowBox(c) => (1, 44 + u16::from(c)),
            Tile::BarrelSide(c) => (2, 44 + u16::from(c)),
            Tile::Barrel(c) => (3, 44 + u16::from(c)),
            Tile::LightStick(c) => (4, 44 + u16::from(c)),
            Tile::Light(c) => (5, 44 + u16::from(c)),
            Tile::Key(c) => (6, 44 + u16::from(c)),
            Tile::Crystal(c) => (7, 44 + u16::from(c)),
            Tile::FlagFrame1(c) => (8, 44 + u16::from(c)),
            Tile::FlagFrame2(c) => (9, 44 + u16::from(c)),
            Tile::FallenFlag(c) => (10, 44 + u16::from(c)),
            Tile::Button { pressed, color } => (11 + u16::from(pressed), 44 + u16::from(color)),
            Tile::Lock(c) => (13, 44 + u16::from(c)),
            Tile::LaserLever(lr, c) => (14 + u16::from(lr), 44 + u16::from(c)),
            Tile::LaserSpark(c) => (16, 44 + u16::from(c)),
            Tile::LaserBeamH(c) => (17, 44 + u16::from(c)),
            Tile::LaserBeamV(c) => (18, 44 + u16::from(c)),
            Tile::TranslucentBarrel(c) => (19, 44 + u16::from(c)),
            Tile::Zapper(lrtb, a) => (
                20 + u16::from(a),
                match lrtb {
                    LRTB::L => 46,
                    LRTB::R => 47,
                    LRTB::T => 44,
                    LRTB::B => 45,
                },
            ),
            Tile::Lever(lmr) => (22 + u16::from(lmr), 45),
            Tile::Chain => (23, 46),
            Tile::Weight => (22, 47),
            Tile::WeightChain => (23, 47),
            Tile::Spring => (22, 44),
            Tile::SpringUp => (23, 44),
            Tile::Grinder(a) => (23 + u16::from(a), 42),
            Tile::HalfGrinder(a) => (23 + u16::from(a), 43),
            Tile::CoinBronze => (22, 41),
            Tile::CoinSilver => (24, 41),
            Tile::CoinGold => (23, 41),
            Tile::SwordBronze => (22, 40),
            Tile::SwordSilver => (24, 40),
            Tile::SwordGold => (23, 40),
            Tile::ShieldBronze => (22, 39),
            Tile::ShieldSilver => (24, 39),
            Tile::ShieldGold => (23, 39),
            Tile::PurpleGun => (22, 37),
            Tile::PurpleGunFiring => (23, 37),
            Tile::SilverGun => (22, 38),
            Tile::SilverGunFiring => (23, 38),
            Tile::Bomb => (24, 38),
            Tile::BombFlash => (24, 37),
            Tile::Chimney(a) => (12 + u16::from(a), 37),
            Tile::SignBed { hanging } => (12, 38 + u16::from(hanging)),
            Tile::SignCoin { hanging } => (13, 38 + u16::from(hanging)),
            Tile::SignMug { hanging } => (14, 38 + u16::from(hanging)),
            Tile::TorchHolder => (15, 37),
            Tile::Torch(lr) => (15, 38 + u16::from(lr)),
            Tile::Umbrella { open } => (16 + u16::from(open), 38),
            Tile::Clock => (16, 37),
            Tile::WeatherVane => (17, 37),
            Tile::Shade(a) => (12 + u16::from(a), 40),
            Tile::RopeVHook => (12, 41),
            Tile::RopeH => (13, 41),
            Tile::RopeV => (14, 41),
            Tile::Fence(a) => (12 + u16::from(a), 42),
            Tile::FenceBroken => (14, 42),
            Tile::FenceLow => (15, 42),
            Tile::FenceOpen => (16, 42),
            Tile::FenceLowerHalf => (17, 42),
            Tile::FenceLower => (18, 42),
            Tile::Bridge => (19, 42),
            Tile::BridgeLog => (20, 42),
            Tile::SignExit => (12, 43),
            Tile::SignArrow(lr) => (13 + u16::from(lr), 43),
            Tile::SignBlank => (15, 43),
            Tile::DoorwayGrey(tb) => (17, 40 + u16::from(tb)),
            Tile::DoorwayBeige(tb) => (21, 40 + u16::from(tb)),
            Tile::DoorLockedGrey(tb) => (16, 40 + u16::from(tb)),
            Tile::DoorLockedBeige(tb) => (20, 40 + u16::from(tb)),
            Tile::DoorInsetBeige(tb) => (19, 40 + u16::from(tb)),
            Tile::DoorTopWindow => (18, 40),
            Tile::DoorBeige => (18, 41),
            Tile::WindowOpenHalf => (16, 39),
            Tile::WindowStainedHalf => (17, 39),
            Tile::WindowOpen => (18, 39),
            Tile::Window(a) => (19 + u16::from(a), 39),
            Tile::HighWindowOpen(tmb) => (18, 36 + u16::from(tmb)),
            Tile::HighWindow(tmb, a) => (19 + u16::from(a), 36 + u16::from(tmb)),
            Tile::CastleWindowOpenSmall(a) => (0 + 2 * u16::from(a), 48),
            Tile::CastleWindowShutSmall(a) => (1 + 2 * u16::from(a), 48),
            Tile::CastleWindowOpen(a) => (0 + 2 * u16::from(a), 49),
            Tile::CastleWindowShut(a) => (1 + 2 * u16::from(a), 49),
            Tile::CastleWindowOpenHigh(a, tmb) => (0 + 2 * u16::from(a), 51 + u16::from(tmb)),
            Tile::CastleWindowShutHigh(a, tmb) => (1 + 2 * u16::from(a), 51 + u16::from(tmb)),
            Tile::CastleWindowOpenSlit(a, tmb) => (0 + 2 * u16::from(a), 55 + u16::from(tmb)),
            Tile::CastleWindowShutSlit(a, tmb) => (1 + 2 * u16::from(a), 55 + u16::from(tmb)),
            Tile::CastleWindowOpenHighAlt(a) => (0 + 2 * u16::from(a), 50),
            Tile::CastleWindowShutHighAlt(a) => (1 + 2 * u16::from(a), 50),
            Tile::CastleWindowOpenSlitAlt(a) => (0 + 2 * u16::from(a), 54),
            Tile::CastleWindowShutSlitAlt(a) => (1 + 2 * u16::from(a), 54),
            Tile::BannerSmallRed(tmb, a) => (4 + 2 * u16::from(a), 48 + u16::from(tmb)),
            Tile::BannerSmallGreen(tmb, a) => (5 + 2 * u16::from(a), 48 + u16::from(tmb)),
            Tile::TapestryMidRed(a) => (4, 51 + u16::from(a)),
            Tile::TapestryMidGreen(a) => (5, 51 + u16::from(a)),
            Tile::TapestryBottomRed(a) => (4, 56 + u16::from(a)),
            Tile::TapestryBottomGreen(a) => (5, 56 + u16::from(a)),
            Tile::BannerRed(a) => (6, 51 + u16::from(a)),
            Tile::BannerGreen(a) => (7, 51 + u16::from(a)),
            Tile::FlagWoodRedFrame1(lr) => (8 + 2 * u16::from(lr), 48),
            Tile::FlagWoodRedFrame2(lr) => (9 + 2 * u16::from(lr), 48),
            Tile::FlagWoodGreenFrame1(lr) => (8 + 2 * u16::from(lr), 51),
            Tile::FlagWoodGreenFrame2(lr) => (9 + 2 * u16::from(lr), 51),
            Tile::FlagWoodLongRedFrame1(lr) => (8 + u16::from(lr), 49 + u16::from(lr)),
            Tile::FlagWoodLongRedFrame2(lr) => (10 + u16::from(lr), 49 + u16::from(lr)),
            Tile::FlagWoodLongGreenFrame1(lr) => (8 + u16::from(lr), 52 + u16::from(lr)),
            Tile::FlagWoodLongGreenFrame2(lr) => (10 + u16::from(lr), 49 + u16::from(lr)),
            Tile::FlagWoodTipRedFrame1(lr) => (9 - u16::from(lr), 49 + u16::from(lr)),
            Tile::FlagWoodTipRedFrame2(lr) => (11 - u16::from(lr), 49 + u16::from(lr)),
            Tile::FlagWoodTipGreenFrame1(lr) => (9 - u16::from(lr), 52 + u16::from(lr)),
            Tile::FlagWoodTipGreenFrame2(lr) => (11 - u16::from(lr), 52 + u16::from(lr)),
            Tile::FlagpoleTop(lmr) => match lmr {
                LMR::L => (8, 54),
                LMR::M => (10, 54),
                LMR::R => (9, 54),
            },
            Tile::Flagpole(lmr) => match lmr {
                LMR::L => (8, 55),
                LMR::M => (10, 55),
                LMR::R => (9, 55),
            },
            Tile::FlagpoleAlt(a) => (10 + u16::from(a), 56),
            Tile::FallenFlagRed(lr) => (8 + u16::from(lr), 56),
            Tile::FallenFlagGreen(lr) => (8 + u16::from(lr), 57),
            Tile::FlagBase(a) => (10 + u16::from(a), 57),
            Tile::TorchWood(lr) => (10 + u16::from(lr), 58),
            Tile::Console { lmr, on, knobs } => (
                match lmr {
                    Some(lmr) => 12 + u16::from(lmr),
                    None if knobs => 16,
                    None => 15,
                },
                48 + u16::from(on),
            ),
            Tile::ConsoleButtons(lmr) => (12 + u16::from(lmr), 50),
            Tile::ConsoleButtonsSmall(lmr) => (12 + u16::from(lmr), 51),
            Tile::Shelves(tb) => (18, 48 + u16::from(tb)),
            Tile::ShelvesNarrow(tb) => (19, 48 + u16::from(tb)),
            Tile::Pillar(tmb) => (20, 48 + u16::from(tmb)),
            Tile::BangSticker => (17, 50),
            Tile::DiamondSticker => (18, 50),
            Tile::CoinSticker => (19, 50),
            Tile::TapeYellow(a) => (16 + u16::from(a), 51),
            Tile::TapeRed(a) => (16 + u16::from(a), 52),
            Tile::CrenellationsBrickTop(lmr) => (12 + u16::from(lmr), 52),
            Tile::CrenellationsBrick(lmr) => (12 + u16::from(lmr), 53),
            Tile::TapestryHolder(lr) => (12 + 2 * u16::from(lr), 56),
            Tile::TapestryTopRed => (13, 56),
            Tile::TapestryTopGreen => (15, 56),
            Tile::CastleStoneWall(a) => (18 + u16::from(a), 54),
            Tile::Castle(a, lmr, tb) => {
                (21 + 3 * u16::from(a) + u16::from(lmr), 53 + u16::from(tb))
            }
            Tile::CastleBeamSlash(a, lmr) => (21 + 3 * u16::from(a) + u16::from(lmr), 52),
            Tile::CastleBeamCross(a, lmr) => (21 + 3 * u16::from(a) + u16::from(lmr), 51),
            Tile::CastleBeamSlats(a, tmb) => (23 + 3 * u16::from(a), 48 + u16::from(tmb)),
            Tile::CastleSlash(a, tmb) => (22 + 3 * u16::from(a), 48 + u16::from(tmb)),
            Tile::CastleBeamSlashV(a, tb) => (21 + 3 * u16::from(a), 48 + 2 * u16::from(tb)),
            Tile::CastleSlat(a) => (21 + 3 * u16::from(a), 49),
            Tile::CastleRoof(a2, a) => (12 + u16::from(a), 57 + u16::from(a2)),
            Tile::CastleRoofPeak(a, lr) => (14 + u16::from(lr), 57 + u16::from(a)),
            Tile::CastleRoofSlope(a, lr) => (16 + u16::from(lr), 57 + u16::from(a)),
            Tile::CastleRoofBend(a, lr) => (18 + u16::from(lr), 57 + u16::from(a)),
            Tile::CastleRoofPeakBase(a, lr) => (20 + u16::from(lr), 57 + u16::from(a)),
            Tile::CastleRoofLow(a, lr) => (22 + u16::from(lr), 57 + u16::from(a)),
            Tile::CastleRoofLowCont(a, lr) => (24 + u16::from(lr), 57 + u16::from(a)),
        }
    }
}

fn range<T: PrimInt + SampleUniform>(gen: &mut Pcg64, range: Extent<T>) -> T {
    if let (Some(lo), Some(hi)) = (range.lo(), range.hi()) {
        gen.gen_range(lo..=hi)
    } else {
        panic!("empty range");
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Rect(pub Place, pub UVec2);

impl Rect {
    fn to_aabb(self) -> AABB<(i32, i32)> {
        AABB::from_corners(
            (self.0.x, self.0.y),
            (self.0.x + self.1.x as i32, self.0.y + self.1.y as i32),
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Sequence, FromPrimitive)]
pub enum GroundCover {
    FullyCovered,
    TopCovered,
    Bare,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Sequence, FromPrimitive)]
pub enum Zone {
    GrassPlains,
    GrassHills,
    GrassLake,
}

#[derive(Clone, Copy, Debug)]
pub enum Feature {
    GroundBlock(GroundCover, Terrain, Rect),
    HillBlock {
        terrain: Terrain,
        start: Place,
        start_height: i32,
        end_height: i32,
    },
    HillBridge {
        terrain: Terrain,
        start: Place,
        start_height: i32,
        end_height: i32,
        thickness: i32,
    },
    Igloo {
        start: Place,
        height: u32,
        width: u32,
        door: u32,
    },
    Tile(Place, Tile),
    BoxBonus(Rect),
    BrickBonus(Rect),
    SurfaceWater(Rect),

    FlatGround(Place, u32),

    Zone(Zone, Rect),
    Offscreen(Rect),
}

impl RTreeObject for Feature {
    type Envelope = AABB<(i32, i32)>;

    fn envelope(&self) -> Self::Envelope {
        match *self {
            Feature::GroundBlock(_, _, r) => r.to_aabb(),
            Feature::HillBlock {
                start,
                start_height,
                end_height,
                ..
            } => AABB::from_corners(
                (start.x, start.y),
                (start.x + end_height - start_height, start.y + end_height),
            ),
            Feature::HillBridge {
                start,
                end_height,
                thickness,
                ..
            } => AABB::from_corners(
                (start.x, start.y),
                (start.x + end_height - thickness, start.y + end_height),
            ),
            Feature::Igloo {
                start,
                height,
                width,
                ..
            } => AABB::from_corners(
                (start.x, start.y),
                (start.x + width as i32, start.y + height as i32),
            ),
            Feature::Tile(start, _) => AABB::from_point((start.x, start.y)),
            Feature::BoxBonus(r) => r.to_aabb(),
            Feature::BrickBonus(r) => r.to_aabb(),
            Feature::SurfaceWater(r) => r.to_aabb(),
            Feature::FlatGround(p, length) => {
                AABB::from_corners((p.x, p.y), (p.x + length as i32, p.y))
            }
            Feature::Zone(_, r) => r.to_aabb(),
            Feature::Offscreen(r) => r.to_aabb(),
        }
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

#[derive(Default, Debug)]
pub struct Schema {
    pub features: RTree<Feature>,
}

#[derive(Resource)]
pub struct Gen {
    pub zone: noise::SuperSimplex,
    pub terrain: noise::SuperSimplex,
    pub theme: noise::Value,
    pub rng: Pcg64,
    pub seed: u128,
}

impl Default for Gen {
    fn default() -> Self {
        let mut tr = thread_rng();
        let mut rng = Pcg64::from_rng(&mut tr).unwrap();
        let terrain = noise::SuperSimplex::new(rng.next_u32());
        let zone = noise::SuperSimplex::new(rng.next_u32());
        let theme = noise::Value::new(rng.next_u32());
        let seed: u128 = rng.gen();

        Self {
            zone,
            terrain,
            theme,
            rng,
            seed,
        }
    }
}

fn n_to_01(n: f64) -> f64 {
    0.5 * (n + 0.1)
}

fn n_to_bool(n: f64) -> bool {
    n_to_01(n).floor() as u32 % 2 == 0
}

fn n_to_enum<T: Sequence + FromPrimitive>(n: f64) -> T {
    T::from_u32(((T::CARDINALITY as f64) * n_to_01(n)).floor() as u32)
        .unwrap_or(T::from_u32(0).unwrap())
}

fn n_to_slice<T: Copy>(n: f64, slice: &[T]) -> T {
    slice[(((slice.len() as f64) * n_to_01(n)).floor()) as usize]
}

fn n_to_range(n: f64, top: u32) -> u32 {
    (n_to_01(n).floor()) as u32 % top
}

pub fn generate_level(gen: &mut Gen) -> Schema {
    let mut features = vec![
        //Feature::Offscreen(Rect(Place::new(-10, -10), UVec2::new(10, 200))),
        //Feature::Offscreen(Rect(Place::new(-10, -10), UVec2::new(200, 10))),
        Feature::Zone(
            Zone::GrassPlains,
            Rect(Place::new(0, 0), UVec2::new(50, 20)),
        ),
    ];

    let mut x = 50;
    for _ in 0..20 {
        let z = n_to_enum(gen.zone.get([x as f64, 0.0]));
        let width = ((Zone::CARDINALITY as f64) * n_to_01(gen.zone.get([x as f64, 0.0]))) % 1.0;
        let width = (50.0 * width).floor() as u32 + 20;
        features.push(Feature::Zone(
            z,
            Rect(Place::new(x, 0), UVec2::new(width, 20)),
        ));
        x += width as i32;
    }

    // features.push(Feature::Offscreen(Rect(
    //     Place::new(x, -10),
    //     UVec2::new(10, 200),
    // )));

    let mut schema = Schema {
        features: RTree::bulk_load(features),
    };

    height_map_floor_brush(gen, &mut schema);
    detect_flat_ground(gen, &mut schema);
    place_bonuses(gen, &mut schema);

    return schema;
}

#[derive(Clone, Copy, Debug)]
struct HeightData {
    height: u32,
    length: u32,
    x: i32,
}

fn height_map_floor(gen: &mut Gen, start: i32, full_length: u32) -> Vec<HeightData> {
    let mut out = Vec::new();

    let mut length = 0;
    let mut previous_height = None;

    for x in 0..full_length {
        let next_height =
            (10.0 * gen.terrain.get([0.1 * (start as f64 + x as f64), 0.0])).floor() as u32;
        if previous_height == Some(next_height) {
            length += 1;
        } else {
            if let Some(prev) = previous_height {
                out.push(HeightData {
                    height: prev,
                    length,
                    x: start + (x - length) as i32,
                });
            }
            previous_height = Some(next_height);
            length = 0;
        }
    }

    if let Some(prev) = previous_height {
        out.push(HeightData {
            height: prev,
            length,
            x: start + (full_length - length) as i32,
        });
    }
    return out;
}

fn gentle_slope(
    gen: &mut Gen,
    schema: &mut Schema,
    terrain: Terrain,
    start_y: i32,
    h0: HeightData,
    h1_x: i32,
    h1_y: u32,
) {
    let diff = h1_y as i32 - h0.height as i32;
    let bridge_or_block = n_to_bool(gen.zone.get([h0.x as f64, 1.0]));

    if diff.abs() > h1_x - h0.x {
        schema.features.insert(Feature::GroundBlock(
            GroundCover::TopCovered,
            terrain,
            Rect(
                Place::new(h0.x, start_y),
                UVec2::new((h1_x - h0.x) as u32, h1_y),
            ),
        ));
    } else {
        schema.features.insert(if bridge_or_block {
            Feature::HillBridge {
                terrain,
                start: Place::new(h0.x, start_y),
                start_height: h0.height as i32,
                end_height: h1_y as i32,
                thickness: 2 + n_to_range(gen.zone.get([h0.x as f64, 0.0]), 7) as i32,
            }
        } else {
            Feature::HillBlock {
                terrain,
                start: Place::new(h0.x, start_y),
                start_height: h0.height as i32,
                end_height: h1_y as i32,
            }
        });
        schema.features.insert(Feature::GroundBlock(
            GroundCover::TopCovered,
            Terrain::Grass,
            Rect(
                Place::new(h0.x + diff.abs(), start_y),
                UVec2::new((h1_x - (h0.x + diff.abs())) as u32, h1_y),
            ),
        ));
    }
}

fn height_map_floor_brush(gen: &mut Gen, schema: &mut Schema) {
    let zones: Vec<(Zone, Rect)> = schema
        .features
        .iter()
        .filter_map(|&f| match f {
            Feature::Zone(z, r) => Some((z, r)),
            _ => None,
        })
        .collect();

    for &(z, r) in zones.iter() {
        let hmap = height_map_floor(gen, r.0.x, r.1.x);
        match z {
            Zone::GrassPlains => {
                for &HeightData { height, length, x } in hmap.iter() {
                    let cover = n_to_enum(gen.zone.get([x as f64, 0.0]));
                    schema.features.insert(Feature::GroundBlock(
                        cover,
                        Terrain::Grass,
                        Rect(Place::new(x, r.0.y), UVec2::new(length, height / 4)),
                    ))
                }
            }
            Zone::GrassHills => {
                for slice in hmap.windows(2) {
                    if let &[h0, h1] = slice {
                        gentle_slope(gen, schema, Terrain::Grass, r.0.y, h0, h1.x, h1.height);
                    }
                }
                let h0 = hmap.last().unwrap();
                let h1 = height_map_floor(gen, h0.x + h0.length as i32, 1)[0];
                schema.features.insert(Feature::GroundBlock(
                    GroundCover::TopCovered,
                    Terrain::Grass,
                    Rect(Place::new(h0.x, r.0.y), UVec2::new(h0.length, h1.height)),
                ))
            }
            Zone::GrassLake => {
                for &HeightData { height, length, x } in hmap.iter() {
                    schema.features.insert(Feature::GroundBlock(
                        GroundCover::TopCovered,
                        Terrain::Grass,
                        Rect(Place::new(x, 0), UVec2::new(length, height)),
                    ))
                }

                schema.features.insert(Feature::SurfaceWater(Rect(
                    r.0,
                    UVec2::new(r.1.x, hmap.first().unwrap().height / 2),
                )));
            }
        }
    }
}

fn detect_flat_ground(gen: &mut Gen, schema: &mut Schema) {
    let ground = schema
        .features
        .iter()
        .filter_map(|&f| match f {
            Feature::GroundBlock(_, _, r) => Some((r.0 + Place::new(0, r.1.y as i32), r.1.x)),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (p, l) in ground {
        schema.features.insert(Feature::FlatGround(p, l))
    }
}

fn place_bonuses(gen: &mut Gen, schema: &mut Schema) {
    let ground = schema
        .features
        .iter()
        .filter_map(|&f| match f {
            Feature::FlatGround(start, l) => Some((start, l)),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (p, l) in ground {
        let run_length = f64::max((l as f64) * n_to_01(gen.zone.get([p.x as f64, 2.0])), 0.0);
        let run_length = run_length.floor() as u32;
        let height = n_to_01(gen.zone.get([p.x as f64, 2.0])) * 4.0 + 2.0;
        let height = height.floor() as u32;
        if run_length != 0 {
            let start = (l / 2) - (run_length / 2);
            schema.features.insert(Feature::BrickBonus(Rect(
                p + Place::new(start as i32, height as i32),
                UVec2::new(run_length, 0),
            )));
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum AbstractTile {
    Exactly(Tile),
    Covering(GroundCover, Terrain),
}

fn get_tile(schema: &Schema, p: (i32, i32)) -> AbstractTile {
    let mut t = AbstractTile::Exactly(Tile::Air);

    for &f in schema.features.locate_all_at_point(&p) {
        match f {
            Feature::GroundBlock(c, terr, _) => t = AbstractTile::Covering(c, terr),
            Feature::HillBlock {
                terrain,
                start,
                start_height,
                end_height,
            } => {
                let top = start.y + start_height + p.0 - start.x;
                if p.1 == top {
                    t = AbstractTile::Exactly(Tile::Terrain(
                        terrain,
                        TerrainTile::Slope(if end_height > start_height {
                            LR::L
                        } else {
                            LR::R
                        }),
                    ));
                } else {
                    t = AbstractTile::Exactly(Tile::Terrain(
                        terrain,
                        TerrainTile::BlockFace(LMR::M, TMB::M),
                    ));
                }
            }
            Feature::HillBridge {
                terrain,
                start,
                start_height,
                end_height,
                thickness,
            } => {
                let top = start.y + start_height + p.0 - start.x;
                let bottom = start.y + start_height + p.0 - start.x - thickness;
                if p.1 == top {
                    t = AbstractTile::Exactly(Tile::Terrain(
                        terrain,
                        TerrainTile::Slope(if end_height > start_height {
                            LR::L
                        } else {
                            LR::R
                        }),
                    ));
                } else if p.1 == bottom {
                    t = AbstractTile::Exactly(Tile::Terrain(
                        terrain,
                        TerrainTile::RockSlope(
                            if end_height > start_height {
                                LR::R
                            } else {
                                LR::L
                            },
                            TB::B,
                        ),
                    ));
                } else if bottom < p.1 && p.1 < top {
                    t = AbstractTile::Exactly(Tile::Terrain(
                        terrain,
                        TerrainTile::BlockFace(LMR::M, TMB::M),
                    ));
                }
            }
            Feature::Igloo {
                start,
                height,
                width,
                door,
            } => {
                let tb = if start.y + height as i32 == p.1 {
                    TB::T
                } else {
                    TB::B
                };
                let lmr = if start.x == p.0 {
                    LMR::L
                } else if start.x + width as i32 == p.0 {
                    LMR::R
                } else {
                    LMR::M
                };
                let door = start.y == 0 && (p.0 - start.x) as u32 == door;
                t = AbstractTile::Exactly(if door {
                    Tile::IglooDoor
                } else {
                    match (tb, lmr) {
                        (TB::T, lmr) => Tile::IglooTop(lmr),
                        (TB::B, LMR::L) => Tile::IglooInterior(true),
                        (TB::B, _) => Tile::IglooInterior(false),
                    }
                });
            }
            Feature::Tile(_, tile) => t = AbstractTile::Exactly(tile),
            Feature::BoxBonus(_) => {
                t = AbstractTile::Exactly(Tile::CoinBox {
                    empty: false,
                    alt: true,
                })
            }
            Feature::BrickBonus(_) => {
                t = AbstractTile::Exactly(Tile::CoinBox {
                    empty: false,
                    alt: false,
                })
            }
            Feature::SurfaceWater(r) => {
                t = if r.0.y + r.1.y as i32 == p.1 {
                    AbstractTile::Exactly(Tile::WaterWave)
                } else {
                    AbstractTile::Exactly(Tile::Water)
                }
            }
            Feature::Offscreen(_) => t = AbstractTile::Exactly(Tile::Gate(LMR::M, TB::B, false)),

            _ => (),
        }
    }

    t
}

pub fn render_level(schema: &Schema) -> ndarray::Array3<Tile> {
    let (lo, hi) = (
        schema.features.root().envelope().lower(),
        schema.features.root().envelope().upper(),
    );
    let (width, height) = ((hi.0 - lo.0) as usize, (hi.1 - lo.1) as usize);
    let mut array = ndarray::Array::from_elem([width + 1, height + 1, 4], Tile::Air);

    for (i, j) in iproduct!(0..width as i32, 0..height as i32) {
        let tl = get_tile(schema, (lo.0 + i - 1, lo.1 + j + 1));
        let tm = get_tile(schema, (lo.0 + i + 0, lo.1 + j + 1));
        let tr = get_tile(schema, (lo.0 + i + 1, lo.1 + j + 1));
        let ml = get_tile(schema, (lo.0 + i - 1, lo.1 + j + 0));
        let mr = get_tile(schema, (lo.0 + i + 1, lo.1 + j + 0));
        let bl = get_tile(schema, (lo.0 + i - 1, lo.1 + j - 1));
        let bm = get_tile(schema, (lo.0 + i + 0, lo.1 + j - 1));
        let br = get_tile(schema, (lo.0 + i + 1, lo.1 + j - 1));

        let this = get_tile(schema, (lo.0 + i, lo.1 + j));

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum Seam {
            Sloped,
            Flat,
            Internal,
            Air,
        }

        fn choose_tile(
            gc: GroundCover,
            terr: Terrain,
            tl: AbstractTile,
            top: AbstractTile,
            left: AbstractTile,
        ) -> (Tile, bool) {
            let tl_flat_possible = match top {
                AbstractTile::Exactly(Tile::Terrain(terr2, TerrainTile::BlockFace(_, _)))
                | AbstractTile::Exactly(Tile::Terrain(terr2, TerrainTile::Slope(LR::R)))
                | AbstractTile::Covering(_, terr2)
                    if terr2 == terr =>
                {
                    false
                }
                _ => true,
            };

            let tl_seam = match tl {
                AbstractTile::Exactly(Tile::Terrain(terr2, TerrainTile::Slope(LR::R)))
                    if terr2 == terr =>
                {
                    Seam::Sloped
                }
                AbstractTile::Exactly(Tile::Terrain(terr2, TerrainTile::BlockFace(_, _)))
                    if terr2 == terr =>
                {
                    Seam::Internal
                }
                AbstractTile::Covering(GroundCover::FullyCovered, terr2)
                    if terr2 == terr && tl_flat_possible =>
                {
                    Seam::Flat
                }
                AbstractTile::Covering(_, terr2) if terr2 == terr => Seam::Internal,
                _ => Seam::Air,
            };
            let top_seam = match top {
                AbstractTile::Exactly(Tile::Terrain(terr2, TerrainTile::Slope(LR::L)))
                    if terr2 == terr =>
                {
                    Seam::Sloped
                }
                AbstractTile::Exactly(Tile::Terrain(terr2, TerrainTile::BlockFace(_, _)))
                    if terr2 == terr =>
                {
                    Seam::Internal
                }
                AbstractTile::Covering(GroundCover::FullyCovered, terr2)
                    if terr2 == terr && (tl_seam != Seam::Internal) =>
                {
                    Seam::Flat
                }
                AbstractTile::Covering(GroundCover::FullyCovered, terr2)
                    if terr2 == terr && (tl_seam == Seam::Internal) =>
                {
                    Seam::Internal
                }
                _ => Seam::Air,
            };
            let left_seam = match left {
                AbstractTile::Exactly(Tile::Terrain(terr2, TerrainTile::Slope(LR::L)))
                    if terr2 == terr =>
                {
                    Seam::Sloped
                }
                AbstractTile::Covering(_, terr2)
                | AbstractTile::Exactly(Tile::Terrain(terr2, TerrainTile::BlockFace(_, _)))
                    if terr2 == terr && tl_seam == Seam::Flat =>
                {
                    Seam::Flat
                }
                AbstractTile::Covering(_, terr2)
                | AbstractTile::Exactly(Tile::Terrain(terr2, TerrainTile::BlockFace(_, _)))
                    if terr2 == terr && tl_seam == Seam::Sloped =>
                {
                    Seam::Sloped
                }
                AbstractTile::Covering(GroundCover::FullyCovered, terr2)
                    if terr2 == terr && tl_seam == Seam::Air =>
                {
                    Seam::Flat
                }
                AbstractTile::Covering(GroundCover::TopCovered, terr2)
                    if terr2 == terr && tl_seam == Seam::Air =>
                {
                    Seam::Flat
                }
                AbstractTile::Covering(GroundCover::Bare, terr2) if terr2 == terr => Seam::Internal,
                AbstractTile::Covering(_, terr2) if terr2 == terr => Seam::Internal,
                AbstractTile::Exactly(Tile::Terrain(terr2, TerrainTile::BlockFace(_, _)))
                    if terr2 == terr =>
                {
                    Seam::Internal
                }
                _ => Seam::Air,
            };

            use Seam::*;
            let (this_seam, end_cap, top_cover, left_cover) = match (top_seam, left_seam) {
                (Sloped, Flat) | (Sloped, Sloped) | (Flat, Sloped) => {
                    (Seam::Sloped, false, false, false)
                }
                (Flat, Flat) => (Seam::Flat, false, false, false),
                (Sloped, Internal) | (Flat, Internal) | (Internal, Internal) => {
                    (Seam::Internal, false, false, false)
                }
                (Internal, Sloped) | (Internal, Flat) => (Seam::Internal, true, false, false),
                (Air, Flat) | (Air, Sloped) => (
                    Seam::Internal,
                    gc == GroundCover::Bare,
                    gc != GroundCover::Bare,
                    false,
                ),
                (Flat, Air) | (Sloped, Air) | (Internal, Air) => (
                    Seam::Internal,
                    false,
                    false,
                    gc == GroundCover::FullyCovered,
                ),
                (Air, Internal) => (Seam::Internal, false, gc != GroundCover::Bare, false),
                (Air, Air) => (
                    Seam::Internal,
                    false,
                    gc != GroundCover::Bare,
                    gc == GroundCover::FullyCovered,
                ),
            };

            let t = if this_seam == Seam::Sloped {
                TerrainTile::SlopeInt(LR::L)
            } else if this_seam == Seam::Flat {
                TerrainTile::FaceInt(LR::L, TB::T)
            } else {
                TerrainTile::BlockFace(
                    if left_cover { LMR::L } else { LMR::M },
                    if top_cover { TMB::T } else { TMB::M },
                )
            };

            (Tile::Terrain(terr, t), end_cap)
        }

        fn flip(t: Tile, lr: bool, tb: bool) -> Tile {
            let flip_lmr = |lmr| match lmr {
                LMR::L if lr => LMR::R,
                LMR::R if lr => LMR::L,
                _ => lmr,
            };
            let flip_tmb = |tmb| match tmb {
                TMB::T if tb => TMB::B,
                TMB::B if tb => TMB::T,
                _ => tmb,
            };
            let flip_lr = |l_r| match l_r {
                LR::L if lr => LR::R,
                LR::R if lr => LR::L,
                _ => l_r,
            };
            let flip_tb = |t_b| match t_b {
                TB::T if tb => TB::B,
                TB::B if tb => TB::T,
                _ => t_b,
            };

            match t {
                Tile::Terrain(terrain, TerrainTile::BlockFace(lmr, tmb)) => Tile::Terrain(
                    terrain,
                    TerrainTile::BlockFace(flip_lmr(lmr), flip_tmb(tmb)),
                ),
                Tile::Terrain(terrain, TerrainTile::BlockLedge(lr)) => {
                    Tile::Terrain(terrain, TerrainTile::BlockLedge(flip_lr(lr)))
                }
                Tile::Terrain(terrain, TerrainTile::Cap(lr)) => {
                    Tile::Terrain(terrain, TerrainTile::Cap(flip_lr(lr)))
                }
                Tile::Terrain(terrain, TerrainTile::FaceInt(lr, tb)) => {
                    Tile::Terrain(terrain, TerrainTile::FaceInt(flip_lr(lr), flip_tb(tb)))
                }
                Tile::Terrain(terrain, TerrainTile::Half(a, lmr)) => {
                    Tile::Terrain(terrain, TerrainTile::Half(a, flip_lmr(lmr)))
                }
                Tile::Terrain(terrain, TerrainTile::OverLedge(lr)) => {
                    Tile::Terrain(terrain, TerrainTile::OverLedge(flip_lr(lr)))
                }
                Tile::Terrain(terrain, TerrainTile::RockSlope(lr, tb)) => {
                    Tile::Terrain(terrain, TerrainTile::RockSlope(flip_lr(lr), flip_tb(tb)))
                }
                Tile::Terrain(terrain, TerrainTile::RoundLedge(lr)) => {
                    Tile::Terrain(terrain, TerrainTile::RoundLedge(flip_lr(lr)))
                }
                Tile::Terrain(terrain, TerrainTile::Slope(lr)) => {
                    Tile::Terrain(terrain, TerrainTile::Slope(flip_lr(lr)))
                }
                Tile::Terrain(terrain, TerrainTile::SlopeInt(lr)) => {
                    Tile::Terrain(terrain, TerrainTile::SlopeInt(flip_lr(lr)))
                }
                Tile::Terrain(terrain, TerrainTile::SlopeLedge(lr)) => {
                    Tile::Terrain(terrain, TerrainTile::SlopeLedge(flip_lr(lr)))
                }
                t => t,
            }
        }

        fn flip_2(t: AbstractTile, lr: bool, tb: bool) -> AbstractTile {
            match t {
                AbstractTile::Exactly(t) => AbstractTile::Exactly(flip(t, lr, tb)),
                cov => cov,
            }
        }

        let main = match this {
            AbstractTile::Exactly(Tile::Terrain(terr, TerrainTile::BlockFace(LMR::M, TMB::M))) => {
                Err((GroundCover::Bare, terr))
            }
            AbstractTile::Covering(gc, terr) => Err((gc, terr)),
            AbstractTile::Exactly(t) => Ok(t),
        };

        let main = match main {
            Ok(t) => (t, None, None),
            Err((gc, terr)) => {
                let tlo = choose_tile(
                    gc,
                    terr,
                    flip_2(tl, false, false),
                    flip_2(tm, false, false),
                    flip_2(ml, false, false),
                );
                let tro = choose_tile(
                    gc,
                    terr,
                    flip_2(tr, true, false),
                    flip_2(tm, true, false),
                    flip_2(mr, true, false),
                );
                let blo = choose_tile(
                    gc,
                    terr,
                    flip_2(bl, false, true),
                    flip_2(bm, false, true),
                    flip_2(ml, false, true),
                );
                let bro = choose_tile(
                    gc,
                    terr,
                    flip_2(br, true, true),
                    flip_2(bm, true, true),
                    flip_2(mr, true, true),
                );

                let (left_cap, right_cap) = (tlo.1, tro.1);

                fn merge(t1: Tile, t2: Tile) -> Tile {
                    let out = match (t1, t2) {
                        (
                            Tile::Terrain(t, TerrainTile::BlockFace(lmr1, tmb1)),
                            Tile::Terrain(_, TerrainTile::BlockFace(lmr2, tmb2)),
                        ) => Tile::Terrain(
                            t,
                            TerrainTile::BlockFace(
                                match (lmr1, lmr2) {
                                    (LMR::M, x) | (x, LMR::M) => x,
                                    _ => lmr1,
                                },
                                match (tmb1, tmb2) {
                                    (TMB::M, x) | (x, TMB::M) => x,
                                    _ => tmb1,
                                },
                            ),
                        ),
                        (Tile::Terrain(_, TerrainTile::SlopeInt(_)), _) => t1,
                        (_, Tile::Terrain(_, TerrainTile::SlopeInt(_))) => t2,
                        (Tile::Terrain(_, TerrainTile::FaceInt(_, _)), _) => t1,
                        (_, Tile::Terrain(_, TerrainTile::FaceInt(_, _))) => t2,
                        _ => t1,
                    };
                    out
                }

                let out = (
                    merge(
                        tlo.0,
                        merge(
                            flip(tro.0, true, false),
                            merge(flip(blo.0, false, true), flip(bro.0, true, true)),
                        ),
                    ),
                    left_cap.then_some(terr),
                    right_cap.then_some(terr),
                );

                out
            }
        };

        array[[i as usize + 1, j as usize + 1, 1]] = main.0;
        if let Some(terrain) = main.1 {
            array[[i as usize + 1, j as usize + 1, 2]] =
                Tile::Terrain(terrain, TerrainTile::Cap(LR::R));
        }
        if let Some(terrain) = main.2 {
            array[[i as usize + 1, j as usize + 1, 3]] =
                Tile::Terrain(terrain, TerrainTile::Cap(LR::L));
        }
    }

    dbg!(array.slice(s![60..100, 0..5, 1]));

    array
}

// pub fn generate_level(gen: &mut Gen, fl: &mut FeatureList) {
//     let mut x = 0;
//     for i in 0..20 {
//         let height = (6.0 * (gen.terrain.get([x as f64, 0.0]) + 1.0)).floor() as u32;
//         let width = gen.rng.gen_range(0..=8);
//         let terrain_type = Terrain::from_u32(((Terrain::CARDINALITY as f64)*0.5*(gen.zone.get([x as f64 / 500.0, 1.0]) + 1.0)).floor() as u32);
//         fl.add(Feature::CoveredGround(
//             terrain_type.unwrap_or(Terrain::Grass), Rect(Place::new(x, 0), UVec2::new(width, height))
//         ));
//         x += width as i32
//     }
// }

// pub fn dress_level(fl: &FeatureList) -> ndarray::Array3<Tile> {
//     let mut arr = ndarray::Array3::from_elem(
//         [fl.boundary.1.x as usize + 2, fl.boundary.1.y as usize + 2, 4], Tile::Air);
//     for &f in fl.features.iter() {
//         match f {
//             Feature::CoveredGround(t, r) => {
//                 for (i, j) in iproduct!(0..r.1.x, 0..r.1.y) {
//                     arr[[
//                         1 + (r.0.x - fl.boundary.0.x + i as i32) as usize,
//                         1 + (r.0.y - fl.boundary.0.y + j as i32) as usize,
//                         1
//                     ]] = Tile::Terrain(t, TerrainTile::BlockFace(LMR::M, TMB::M));
//                 }
//             }
//         }
//     }

//     let mut arr2 = ndarray::Array3::from_elem(
//         [fl.boundary.1.x as usize + 2, fl.boundary.1.y as usize + 2, 4], Tile::Air);
//     let mut arr2c = arr2.slice_mut(ndarray::s!(1..-1, 1..-1, ..));
//     ndarray::Zip::indexed(arr.windows([3, 3, 1])).for_each(|(i, j, k), s_| {
//         let s = s_.index_axis(ndarray::Axis(2), 0);

//         fn int_block(t: Tile) -> Option<Terrain> {
//             if let Tile::Terrain(style, TerrainTile::BlockFace(LMR::M, TMB::M)) = t {
//                 Some(style)
//             } else {
//                 None
//             }
//         }

//         if let Some(style) = int_block(s[[1, 1]]) {
//             let top = int_block(s[[1, 2]]) == Some(style);
//             let bottom = int_block(s[[1, 0]]) == Some(style);
//             let left = int_block(s[[0, 1]]) == Some(style);
//             let right = int_block(s[[2, 1]]) == Some(style);
//             let tl = int_block(s[[0, 2]]) == Some(style);
//             let tr = int_block(s[[2, 2]]) == Some(style);
//             let bl = int_block(s[[0, 0]]) == Some(style);
//             let br = int_block(s[[2, 0]]) == Some(style);

//             let lmr = match (left, right) {
//                 (true, true) => Some(LMR::M),
//                 (true, false) => Some(LMR::R),
//                 (false, true) => Some(LMR::L),
//                 (false, false) => None
//             };
//             let tmb = match (top, bottom) {
//                 (true, true) => Some(TMB::M),
//                 (true, false) => Some(TMB::B),
//                 (false, true) => Some(TMB::T),
//                 (false, false) => None
//             };
//             let tt = match (lmr, tmb) {
//                 (Some(LMR::M), Some(TMB::M)) => {
//                     if !tl as u32 + !tr as u32 + !bl as u32 + !br as u32 > 1 {
//                         TerrainTile::Jagged
//                     } else if !tl { TerrainTile::FaceInt(LR::L, TB::T) }
//                     else if !tr { TerrainTile::FaceInt(LR::R, TB::T) }
//                     else if !bl { TerrainTile::FaceInt(LR::L, TB::B) }
//                     else if !br { TerrainTile::FaceInt(LR::R, TB::B) }
//                     else { TerrainTile::BlockFace(LMR::M, TMB::M) }
//                 },
//                 (Some(lmr), Some(tmb)) => TerrainTile::BlockFace(lmr, tmb),
//                 (None, Some(TMB::T)) => TerrainTile::Single,
//                 (None, Some(_)) => TerrainTile::BlockFace(LMR::M, TMB::M),
//                 (Some(LMR::L), None) => TerrainTile::BlockLedge(LR::L),
//                 (Some(LMR::M), None) =>  TerrainTile::BlockFace(LMR::M, TMB::T),
//                 (Some(LMR::R), None) => TerrainTile::BlockLedge(LR::R),
//                 (None, None) => TerrainTile::Block
//             };

//             arr2c[[i, j, 1]] = Tile::Terrain(style, tt);
//         }
//     });
//     arr2
// }

// DEPRECATED
// fn pick_extent(ground: Extent<i32>, bounds: Option<Extent<i32>>, gen: &mut Pcg64) -> Extent<i32> {
//     if let (Some(lo), Some(hi)) = (ground.lo(), ground.hi()) {
//         let min_width = bounds.and_then(|e| e.lo()).unwrap_or(0);
//         let max_width = bounds.and_then(|e| e.hi()).unwrap_or(ground.len());
//         let width = gen.gen_range(min_width..=max_width);
//         let start = gen.gen_range(lo..=hi - width);
//         let end = start + width;
//         Extent::new(start, end)
//     } else {
//         ground
//     }
// }

// pub fn igloo(ground: Extent<i32>, height: i32, gen: &mut Pcg64) -> Vec<(Place, Tile)> {
//     let extent = pick_extent(ground, Some(Extent::new(2, 4)), gen);
//     let d_height = i32::min(extent.len(), 4);
//     let top = gen.gen_range(height + 1..height + d_height);
//     let door = range(gen, extent);

//     let mut v = Vec::new();
//     for (i, j) in iproduct!(
//         extent.iter().with_position(),
//         Extent::new(height, top).iter().with_position()
//     ) {
//         let t = match (i, j) {
//             (First(i), Last(j)) => Tile::IglooTop(LMR::L),
//             (Middle(i), Last(j)) => Tile::IglooTop(LMR::M),
//             (Last(i), Last(j)) => Tile::IglooTop(LMR::R),
//             _ => {
//                 if i.into_inner() == door && j.into_inner() == height {
//                     Tile::IglooDoor
//                 } else {
//                     *[Tile::IglooInterior(false), Tile::IglooInterior(true)]
//                         .choose(gen)
//                         .unwrap()
//                 }
//             }
//         };
//         v.push((Place::new(i.into_inner(), j.into_inner()), t));
//     }
//     v
// }

// pub fn heightmap_ground(start: Place, gen: &mut Pcg64, s: OpenSimplex) -> Vec<(Place, Tile)> {
//     let mut heights = Vec::new();

//     let shift = |x| 1.0 + ((x + 1.0) / 2.0) * 10.0;

//     for i in 0..CHUNK_SIZE {
//         heights.push(shift(s.get([start.x as f64 + i as f64, start.y as f64])));
//     }

//     let mut v = Vec::new();
//     for (i, h) in heights.iter().copied().enumerate() {
//         let h_ = h.ceil() as usize;
//         for j in 0..h_ {
//             let tile = if j == h_ - 1 {
//                 TerrainTile::BlockFace(LMR::M, TMB::T)
//             } else {
//                 TerrainTile::BlockFace(LMR::M, TMB::M)
//             };
//             v.push((
//                 Place::new( i as i32, j as i32),
//                 Tile::Terrain(Terrain::Dirt, tile),
//             ));
//         }
//     }

//     v
// }

// pub fn slopey_ground(start: Place, gen: &mut Pcg64, s: OpenSimplex) -> Vec<(Place, Tile)> {
//     #[derive(Clone, Copy, PartialEq, Eq)]
//     enum Angle {
//         A0,
//         A45,
//         A90,
//         N45,
//         N90,
//     }

//     let mut heights = Vec::new();
//     let mut height = 0;
//     let weights: WeightedIndex<usize> = WeightedIndex::new(&[15, 3, 1, 3, 1]).unwrap();

//     for _ in 0..=8 {
//         let length: usize = gen.gen_range(1..=5);
//         let angle =
//             [Angle::A0, Angle::A45, Angle::A90, Angle::N45, Angle::N90][weights.sample(gen)];

//         if heights.len() >= CHUNK_SIZE {
//             break;
//         }

//         for _ in 0..length {
//             height += match angle {
//                 Angle::A0 => 0,
//                 Angle::A45 => 1,
//                 Angle::A90 => 3,
//                 Angle::N45 => -1,
//                 Angle::N90 => -3,
//             };
//             height = 0.max(height);
//             heights.push((height, angle));
//         }
//     }

//     let mut v = Vec::new();
//     for (i, (h, a)) in heights.iter().copied().enumerate() {
//         for j in 0..h {
//             let tile = if j == h - 1 {
//                 match a {
//                     Angle::A45 => TerrainTile::Slope(LR::L),
//                     Angle::N45 => TerrainTile::Slope(LR::R),
//                     _ => TerrainTile::BlockFace(LMR::M, TMB::T),
//                 }
//             } else if j == h - 2 && a == Angle::A45 {
//                 TerrainTile::SlopeInt(LR::L)
//             } else if j == h - 2 && a == Angle::N45 {
//                 TerrainTile::SlopeInt(LR::R)
//             } else {
//                 TerrainTile::BlockFace(LMR::M, TMB::M)
//             };
//             v.push((
//                 Place::new(i as i32, j),
//                 Tile::Terrain(Terrain::Dirt, tile),
//             ));
//         }
//     }

//     v
// }
