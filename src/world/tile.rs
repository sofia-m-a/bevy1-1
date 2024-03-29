use enum_iterator::Sequence;
use itertools::iproduct;
use num_derive::FromPrimitive;

use crate::helpers::*;

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

impl LR {
    pub fn flip(self) -> Self {
        match self {
            LR::L => LR::R,
            LR::R => LR::L,
        }
    }
}

impl From<LR> for u16 {
    fn from(lr: LR) -> Self {
        match lr {
            LR::L => 0,
            LR::R => 1,
        }
    }
}

impl From<LR> for i32 {
    fn from(lr: LR) -> Self {
        match lr {
            LR::L => -1,
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

impl LMR {
    pub fn flip(self) -> Self {
        match self {
            LMR::L => LMR::R,
            LMR::M => LMR::M,
            LMR::R => LMR::L,
        }
    }
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

impl From<LMR> for i32 {
    fn from(lmr: LMR) -> Self {
        match lmr {
            LMR::L => -1,
            LMR::M => 0,
            LMR::R => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TB {
    T,
    B,
}

impl TB {
    pub fn flip(self) -> Self {
        match self {
            TB::T => TB::B,
            TB::B => TB::T,
        }
    }
}

impl From<TB> for u16 {
    fn from(tb: TB) -> Self {
        match tb {
            TB::T => 0,
            TB::B => 1,
        }
    }
}

impl From<TB> for i32 {
    fn from(tb: TB) -> Self {
        match tb {
            TB::T => 1,
            TB::B => -1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TMB {
    T,
    M,
    B,
}

impl TMB {
    pub fn flip(self) -> Self {
        match self {
            TMB::T => TMB::B,
            TMB::M => TMB::M,
            TMB::B => TMB::T,
        }
    }
}

impl From<TMB> for u16 {
    fn from(tmb: TMB) -> Self {
        match tmb {
            TMB::T => 0,
            TMB::M => 1,
            TMB::B => 2,
        }
    }
}

impl From<TMB> for i32 {
    fn from(tmb: TMB) -> Self {
        match tmb {
            TMB::T => 1,
            TMB::M => 0,
            TMB::B => -1,
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

impl LRTB {
    pub fn flip(self) -> Self {
        match self {
            LRTB::L => LRTB::R,
            LRTB::R => LRTB::L,
            LRTB::T => LRTB::B,
            LRTB::B => LRTB::T,
        }
    }
}

pub fn lmr_of(b: Box1<i32>, p: i32) -> LMR {
    if p == b.lo_incl {
        LMR::L
    } else if p == b.hi_excl - 1 {
        LMR::R
    } else {
        LMR::M
    }
}

pub fn tmb_of(b: Box1<i32>, p: i32) -> TMB {
    if p == b.lo_incl {
        TMB::B
    } else if p == b.hi_excl - 1 {
        TMB::T
    } else {
        TMB::M
    }
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Sequence, FromPrimitive)]
pub enum MushroomStyle {
    Caramel,
    Brown,
    Red,
    White,
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

    MushroomBlock(MushroomStyle, Alt, LMR),
    MushroomStemBlock(MushroomStyle, Alt),
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
            Tile::MushroomBlock(MushroomStyle::Caramel, a, lmr) => {
                (12 + u16::from(lmr), 20 + u16::from(a))
            }
            Tile::MushroomStemBlock(MushroomStyle::Caramel, a) => (15, 20 + u16::from(a)),
            Tile::MushroomBlock(MushroomStyle::Brown, a, lmr) => {
                (12 + u16::from(lmr), 22 + u16::from(a))
            }
            Tile::MushroomStemBlock(MushroomStyle::Brown, a) => (15, 22 + u16::from(a)),
            Tile::MushroomBlock(MushroomStyle::Red, a, lmr) => {
                (12 + u16::from(lmr), 24 + u16::from(a))
            }
            Tile::MushroomStemBlock(MushroomStyle::Red, a) => (15, 24 + u16::from(a)),
            Tile::MushroomBlock(MushroomStyle::White, a, lmr) => {
                (12 + u16::from(lmr), 26 + u16::from(a))
            }
            Tile::MushroomStemBlock(MushroomStyle::White, a) => (15, 26 + u16::from(a)),
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Sequence, FromPrimitive)]
pub enum GroundCover {
    FullyCovered,
    TopCovered,
    Bare,
}

#[derive(Clone, Copy, Debug)]
pub enum TilingTile {
    Exactly(Tile),
    Ground(GroundCover, Terrain),
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
enum TilingCorner {
    #[default]
    None,
    Inner,
    Slope,
}

impl TilingCorner {
    fn merge_corners(a: TilingCorner, b: TilingCorner) -> TilingCorner {
        use TilingCorner::*;
        match (a, b) {
            (Slope, _) | (_, Slope) => Slope,
            (Inner, _) | (_, Inner) => Inner,
            (None, None) => None,
        }
    }

    fn is_none(self) -> bool {
        self == TilingCorner::None
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
enum TilingSide {
    #[default]
    Exterior,
    Interior,
}

#[derive(Clone, Copy, Debug)]
struct TileTilingInfo {
    tl: TilingCorner,
    tr: TilingCorner,
    rt: TilingCorner,
    rb: TilingCorner,
    br: TilingCorner,
    bl: TilingCorner,
    lb: TilingCorner,
    lt: TilingCorner,
    left: TilingSide,
    right: TilingSide,
    top: TilingSide,
    bottom: TilingSide,
    terrain: Terrain,
}

impl TileTilingInfo {
    fn new(terrain: Terrain) -> Self {
        Self {
            tl: TilingCorner::None,
            tr: TilingCorner::None,
            rt: TilingCorner::None,
            rb: TilingCorner::None,
            br: TilingCorner::None,
            bl: TilingCorner::None,
            lb: TilingCorner::None,
            lt: TilingCorner::None,
            left: TilingSide::Exterior,
            right: TilingSide::Exterior,
            top: TilingSide::Exterior,
            bottom: TilingSide::Exterior,
            terrain,
        }
    }
}

impl TilingTile {
    fn info(self) -> Option<TileTilingInfo> {
        match self {
            TilingTile::Exactly(Tile::Terrain(terrain, tt)) => Some({
                let mut info = TileTilingInfo::new(terrain);

                match tt {
                    TerrainTile::BlockFace(lmr, tmb) => {
                        info.top = if tmb == TMB::T {
                            TilingSide::Exterior
                        } else {
                            TilingSide::Interior
                        };
                        info.left = if lmr == LMR::L {
                            TilingSide::Exterior
                        } else {
                            TilingSide::Interior
                        };
                        info.right = if lmr == LMR::R {
                            TilingSide::Exterior
                        } else {
                            TilingSide::Interior
                        };
                        info.bottom = if tmb == TMB::B {
                            TilingSide::Exterior
                        } else {
                            TilingSide::Interior
                        };
                        info.tl = if info.top == TilingSide::Interior && lmr == LMR::L {
                            TilingCorner::Inner
                        } else {
                            TilingCorner::None
                        };
                        info.tr = if info.top == TilingSide::Interior && lmr == LMR::R {
                            TilingCorner::Inner
                        } else {
                            TilingCorner::None
                        };
                        info.rt = if info.right == TilingSide::Interior && tmb == TMB::T {
                            TilingCorner::Inner
                        } else {
                            TilingCorner::None
                        };
                        info.rb = if info.right == TilingSide::Interior && tmb == TMB::B {
                            TilingCorner::Inner
                        } else {
                            TilingCorner::None
                        };
                        info.br = if info.bottom == TilingSide::Interior && lmr == LMR::R {
                            TilingCorner::Inner
                        } else {
                            TilingCorner::None
                        };
                        info.bl = if info.bottom == TilingSide::Interior && lmr == LMR::L {
                            TilingCorner::Inner
                        } else {
                            TilingCorner::None
                        };
                        info.lb = if info.left == TilingSide::Interior && tmb == TMB::B {
                            TilingCorner::Inner
                        } else {
                            TilingCorner::None
                        };
                        info.lt = if info.left == TilingSide::Interior && tmb == TMB::T {
                            TilingCorner::Inner
                        } else {
                            TilingCorner::None
                        };
                    }
                    TerrainTile::Slope(lr) => {
                        info.bottom = TilingSide::Interior;
                        match lr {
                            LR::L => {
                                info.right = TilingSide::Interior;
                                info.rt = TilingCorner::Slope;
                                info.bl = TilingCorner::Slope;
                            }
                            LR::R => {
                                info.left = TilingSide::Interior;
                                info.lt = TilingCorner::Slope;
                                info.br = TilingCorner::Slope;
                            }
                        }
                    }
                    TerrainTile::SlopeInt(lr) => {
                        info.top = TilingSide::Interior;
                        info.left = TilingSide::Interior;
                        info.right = TilingSide::Interior;
                        info.bottom = TilingSide::Interior;
                        match lr {
                            LR::L => {
                                info.tl = TilingCorner::Slope;
                                info.lt = TilingCorner::Slope;
                            }
                            LR::R => {
                                info.tr = TilingCorner::Slope;
                                info.rt = TilingCorner::Slope;
                            }
                        }
                    }
                    TerrainTile::FaceInt(lr, tb) => {
                        info.top = TilingSide::Interior;
                        info.left = TilingSide::Interior;
                        info.right = TilingSide::Interior;
                        info.bottom = TilingSide::Interior;
                        match (lr, tb) {
                            (LR::L, TB::T) => {
                                info.bl = TilingCorner::Inner;
                                info.lb = TilingCorner::Inner;
                            }
                            (LR::L, TB::B) => {
                                info.tl = TilingCorner::Inner;
                                info.lt = TilingCorner::Inner;
                            }
                            (LR::R, TB::T) => {
                                info.br = TilingCorner::Inner;
                                info.rb = TilingCorner::Inner;
                            }
                            (LR::R, TB::B) => {
                                info.tr = TilingCorner::Inner;
                                info.rt = TilingCorner::Inner;
                            }
                        }
                    }
                    TerrainTile::Single => {
                        info.bottom = TilingSide::Interior;
                    }
                    TerrainTile::SingleBare => {
                        info.bottom = TilingSide::Interior;
                        info.left = TilingSide::Interior;
                        info.right = TilingSide::Interior;
                    }
                    TerrainTile::Jagged => {
                        info.top = TilingSide::Interior;
                        info.left = TilingSide::Interior;
                        info.right = TilingSide::Interior;
                    }
                    _ => (),
                }
                info
            }),
            TilingTile::Exactly(_) => None,
            TilingTile::Ground(_, terrain) => Some({
                TileTilingInfo {
                    top: TilingSide::Interior,
                    left: TilingSide::Interior,
                    right: TilingSide::Interior,
                    bottom: TilingSide::Interior,
                    tl: TilingCorner::None,
                    tr: TilingCorner::None,
                    rt: TilingCorner::None,
                    rb: TilingCorner::None,
                    br: TilingCorner::None,
                    bl: TilingCorner::None,
                    lb: TilingCorner::None,
                    lt: TilingCorner::None,
                    terrain,
                }
            }),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CapInfo {
    pub left_cap: Option<Terrain>,
    pub right_cap: Option<Terrain>,
}

// Takes in an array extended by one tile on each side to compute the correct output
pub fn compute_tiling(array: ndarray::Array2<TilingTile>) -> ndarray::Array2<(Tile, CapInfo)> {
    let sx = array.shape()[0];
    let sy = array.shape()[1];
    let mut with_sides = array.map(|t| (*t, t.info()));
    let info = with_sides.map(|t| t.1);

    for (i, j) in iproduct!(0..(sx - 2), 0..(sy - 2)) {
        match (array[[i + 1, j + 1]], info[[i + 1, j + 1]]) {
            (TilingTile::Ground(gc, terrain), _) => {
                let left = info[[i, j + 1]]
                    .and_then(|ti| (ti.right == TilingSide::Interior).then_some(ti.terrain));
                let right = info[[i + 2, j + 1]]
                    .and_then(|ti| (ti.left == TilingSide::Interior).then_some(ti.terrain));
                let top = info[[i + 1, j + 2]]
                    .and_then(|ti| (ti.bottom == TilingSide::Interior).then_some(ti.terrain));
                let bottom = info[[i + 1, j]]
                    .and_then(|ti| (ti.top == TilingSide::Interior).then_some(ti.terrain));

                let left =
                    left != Some(terrain) && (gc == GroundCover::FullyCovered || left.is_some());
                let right =
                    right != Some(terrain) && (gc == GroundCover::FullyCovered || right.is_some());
                let top = top != Some(terrain) && (gc != GroundCover::Bare || top.is_some());
                let bottom = bottom != Some(terrain)
                    && (gc == GroundCover::FullyCovered || bottom.is_some());

                let lmr = match (left, right) {
                    (true, _) => LMR::L,
                    (_, true) => LMR::R,
                    (_, _) => LMR::M,
                };
                let tmb = match (top, bottom) {
                    (true, _) => TMB::T,
                    (_, true) => TMB::B,
                    (_, _) => TMB::M,
                };
                if lmr != LMR::M || tmb != TMB::M {
                    let t = TilingTile::Exactly(Tile::Terrain(
                        terrain,
                        TerrainTile::BlockFace(lmr, tmb),
                    ));
                    with_sides[[i + 1, j + 1]] = (t, t.info());
                }
            }
            _ => (),
        }
    }

    let mut out = ndarray::Array2::from_elem(
        [sx - 2, sy - 2],
        (
            Tile::Air,
            CapInfo {
                left_cap: None,
                right_cap: None,
            },
        ),
    );

    for (i, j) in iproduct!(0..(sx - 2), 0..(sy - 2)) {
        match with_sides[[i + 1, j + 1]].0 {
            TilingTile::Exactly(t) => out[[i, j]].0 = t,
            TilingTile::Ground(_, terrain) => {
                let left = with_sides[[i, j + 1]];
                let right = with_sides[[i + 2, j + 1]];
                let top = with_sides[[i + 1, j + 2]];
                let bottom = with_sides[[i + 1, j]];

                let tl = TilingCorner::merge_corners(
                    left.1.map(|ti| ti.rt).unwrap_or(TilingCorner::None),
                    top.1.map(|ti| ti.bl).unwrap_or(TilingCorner::None),
                );
                let tr = TilingCorner::merge_corners(
                    right.1.map(|ti| ti.lt).unwrap_or(TilingCorner::None),
                    top.1.map(|ti| ti.br).unwrap_or(TilingCorner::None),
                );
                let bl = TilingCorner::merge_corners(
                    left.1.map(|ti| ti.rb).unwrap_or(TilingCorner::None),
                    bottom.1.map(|ti| ti.tl).unwrap_or(TilingCorner::None),
                );
                let br = TilingCorner::merge_corners(
                    right.1.map(|ti| ti.lb).unwrap_or(TilingCorner::None),
                    bottom.1.map(|ti| ti.tr).unwrap_or(TilingCorner::None),
                );

                let tt = if tl == TilingCorner::Slope {
                    TerrainTile::SlopeInt(LR::L)
                } else if tr == TilingCorner::Slope {
                    TerrainTile::SlopeInt(LR::R)
                } else {
                    match (tl, tr, bl, br) {
                        (TilingCorner::Inner, _, _, _) => TerrainTile::FaceInt(LR::L, TB::T),
                        (_, TilingCorner::Inner, _, _) => TerrainTile::FaceInt(LR::R, TB::T),
                        (_, _, TilingCorner::Inner, _) => TerrainTile::FaceInt(LR::L, TB::B),
                        (_, _, _, TilingCorner::Inner) => TerrainTile::FaceInt(LR::R, TB::B),
                        _ => TerrainTile::BlockFace(LMR::M, TMB::M),
                    }
                };
                out[[i, j]].0 = Tile::Terrain(terrain, tt)
            }
        }

        out[[i, j]].1 = {
            let right_cap = match (
                with_sides[[i, j + 1]].1,
                TilingTile::Exactly(out[[i, j]].0).info(),
            ) {
                (None, _) => None,
                (Some(ti1), this) => match (ti1.rt.is_none(), this.map(|ti| ti.lt.is_none())) {
                    (true, _) => None,
                    (false, Some(false)) => None,
                    (false, _) => Some(ti1.terrain),
                },
            };
            let left_cap = match (
                with_sides[[i + 2, j + 1]].1,
                TilingTile::Exactly(out[[i, j]].0).info(),
            ) {
                (None, _) => None,
                (Some(ti1), this) => match (ti1.lt.is_none(), this.map(|ti| ti.rt.is_none())) {
                    (true, _) => None,
                    (false, Some(false)) => None,
                    (false, _) => Some(ti1.terrain),
                },
            };
            CapInfo {
                left_cap,
                right_cap,
            }
        }
    }

    out
}
