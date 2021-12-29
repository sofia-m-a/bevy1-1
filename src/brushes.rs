use crate::assets::*;

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
}

pub fn ground(t: GroundTileType, s: GroundSet) -> Tile {
    use Tile::*;
    (match s {
        GroundSet::Grass => [
            GrasscenterRounded,
            Grass,
            Grassleft,
            Grassmid,
            Grassright,
            Grasscliffleftalt,
            Grasscliffrightalt,
            Grasscliffleft,
            Grasscliffright,
            Grasscenter,
            Grasshillright,
            Grasshillleft,
            Grasshillright2,
            Grasshillleft2,
            Grasshalf,
            Grasshalfleft,
            Grasshalfmid,
            Grasshalfright,
            Grassledgeleft,
            Grassledgeright,
        ],
        GroundSet::Sand => [
            SandcenterRounded,
            Sand,
            Sandleft,
            Sandmid,
            Sandright,
            Sandcliffleftalt,
            Sandcliffrightalt,
            Sandcliffleft,
            Sandcliffright,
            Sandcenter,
            Sandhillright,
            Sandhillleft,
            Sandhillright2,
            Sandhillleft2,
            Sandhalf,
            Sandhalfleft,
            Sandhalfmid,
            Sandhalfright,
            Sandledgeleft,
            Sandledgeright,
        ],
        GroundSet::Snow => [
            SnowcenterRounded,
            Snow,
            Snowleft,
            Snowmid,
            Snowright,
            Snowcliffleftalt,
            Snowcliffrightalt,
            Snowcliffleft,
            Snowcliffright,
            Snowcenter,
            Snowhillright,
            Snowhillleft,
            Snowhillright2,
            Snowhillleft2,
            Snowhalf,
            Snowhalfleft,
            Snowhalfmid,
            Snowhalfright,
            Snowledgeleft,
            Snowledgeright,
        ],
        GroundSet::Stone => [
            StonecenterRounded,
            Stone,
            Stoneleft,
            Stonemid,
            Stoneright,
            Stonecliffleftalt,
            Stonecliffrightalt,
            Stonecliffleft,
            Stonecliffright,
            Stonecenter,
            Rockhillright,
            Rockhillleft,
            Stonehillright2,
            Stonehillleft2,
            Stonehalf,
            Stonehalfleft,
            Stonehalfmid,
            Stonehalfright,
            Stoneledgeleft,
            Stoneledgeright,
        ],
        GroundSet::Dirt => [
            DirtcenterRounded,
            Dirt,
            Dirtleft,
            Dirtmid,
            Dirtright,
            Dirtcliffleftalt,
            Dirtcliffrightalt,
            Dirtcliffleft,
            Dirtcliffright,
            Dirtcenter,
            Dirthillright,
            Dirthillleft,
            Dirthillright2,
            Dirthillleft2,
            Dirthalf,
            Dirthalfleft,
            Dirthalfmid,
            Dirthalfright,
            Dirtledgeleft,
            Dirtledgeright,
        ],
        GroundSet::Castle => [
            CastlecenterRounded,
            Castle,
            Castleleft,
            Castlemid,
            Castleright,
            Castlecliffleftalt,
            Castlecliffrightalt,
            Castlecliffleft,
            Castlecliffright,
            Castlecenter,
            Castlehillright,
            Castlehillleft,
            Castlehillright2,
            Castlehillleft2,
            Castlehalf,
            Castlehalfleft,
            Castlehalfmid,
            Castlehalfright,
            Castleledgeleft,
            Castleledgeright,
        ],
        GroundSet::Cake => [
            CakecenterRounded,
            Cake,
            Cakeleft,
            Cakemid,
            Cakeright,
            Cakecliffleftalt,
            Cakecliffrightalt,
            Cakecliffleft,
            Cakecliffright,
            Cakecenter,
            Cakehillright,
            Cakehillleft,
            Cakehillright2,
            Cakehillleft2,
            Cakehalf,
            Cakehalfleft,
            Cakehalfmid,
            Cakehalfright,
            Cakeledgeleft,
            Cakeledgeright,
        ],
        GroundSet::Choco => [
            ChococenterRounded,
            Choco,
            Chocoleft,
            Chocomid,
            Chocoright,
            Chococliffleftalt,
            Chococliffrightalt,
            Chococliffleft,
            Chococliffright,
            Chococenter,
            Chocohillright,
            Chocohillleft,
            Chocohillright2,
            Chocohillleft2,
            Chocohalf,
            Chocohalfleft,
            Chocohalfmid,
            Chocohalfright,
            Chocoledgeleft,
            Chocoledgeright,
        ],
        GroundSet::Tundra => [
            TundracenterRounded,
            Tundra,
            Tundraleft,
            Tundramid,
            Tundraright,
            Tundracliffleftalt,
            Tundracliffrightalt,
            Tundracliffleft,
            Tundracliffright,
            Tundracenter,
            Tundrahillright,
            Tundrahillleft,
            Tundrahillright2,
            Tundrahillleft2,
            Tundrahalf,
            Tundrahalfleft,
            Tundrahalfmid,
            Tundrahalfright,
            Tundraledgeleft,
            Tundraledgeright,
        ],
    })[t as usize]
}

pub fn ground_alt(t: Tile) -> Tile {
    use Tile::*;
    match t {
        Cakehalf => Cakehalfalt,
        Cakehalfleft => Cakehalfaltleft,
        Cakehalfmid => Cakehalfaltmid,
        Cakehalfright => Cakehalfaltright,
        Chocohalf => Chocohalfalt,
        Chocohalfleft => Chocohalfaltleft,
        Chocohalfmid => Chocohalfaltmid,
        Chocohalfright => Chocohalfaltright,
        _ => t,
    }
}
