use std::ops::RangeInclusive;
use std::time::Duration;

use benimator::SpriteSheetAnimation;
use bevy::prelude::*;
use bevy::sprite::Rect;

pub const PLAYER_DUCK: usize = 0;
pub const PLAYER_FRONT: usize = 1;
pub const PLAYER_HURT: usize = 2;
pub const PLAYER_JUMP: usize = 3;
pub const PLAYER_STAND: usize = 4;
const PLAYER_WALK: RangeInclusive<u32> = 5..=15;

const P1_SHEET: &'static str = "Platformer Art Complete Pack/Base pack/Player/p1_spritesheet.png";

#[rustfmt::skip]
const P1_RECTS: [[f32; 4]; 16] = [
    [365.0,   98.0,  69.0,  71.0,], // p1_duck  
    [  0.0,  196.0,  66.0,  92.0,], // p1_front 
    [438.0,    0.0,  69.0,  92.0,], // p1_hurt  
    [438.0,   93.0,  67.0,  94.0,], // p1_jump  
    [ 67.0,  196.0,  66.0,  92.0,], // p1_stand 
    [  0.0,    0.0,  72.0,  97.0,], // p1_walk01
    [ 73.0,    0.0,  72.0,  97.0,], // p1_walk02
    [146.0,    0.0,  72.0,  97.0,], // p1_walk03
    [  0.0,   98.0,  72.0,  97.0,], // p1_walk04
    [ 73.0,   98.0,  72.0,  97.0,], // p1_walk05
    [146.0,   98.0,  72.0,  97.0,], // p1_walk06
    [219.0,    0.0,  72.0,  97.0,], // p1_walk07
    [292.0,    0.0,  72.0,  97.0,], // p1_walk08
    [219.0,   98.0,  72.0,  97.0,], // p1_walk09
    [365.0,    0.0,  72.0,  97.0,], // p1_walk10
    [292.0,   98.0,  72.0,  97.0,], // p1_walk11
];
const P2_SHEET: &'static str = "Platformer Art Complete Pack/Base pack/Player/p2_spritesheet.png";

#[rustfmt::skip]
const P2_RECTS: [[f32; 4]; 16] = [
    [355.0,   95.0,  67.0,  72.0], // p2_duck  
    [  0.0,  190.0,  66.0,  92.0], // p2_front 
    [426.0,    0.0,  67.0,  92.0], // p2_hurt  
    [423.0,   95.0,  66.0,  94.0], // p2_jump  
    [ 67.0,  190.0,  66.0,  92.0], // p2_stand 
    [  0.0,    0.0,  70.0,  94.0], // p2_walk01
    [ 71.0,    0.0,  70.0,  94.0], // p2_walk02
    [142.0,    0.0,  70.0,  94.0], // p2_walk03
    [  0.0,   95.0,  70.0,  94.0], // p2_walk04
    [ 71.0,   95.0,  70.0,  94.0], // p2_walk05
    [142.0,   95.0,  70.0,  94.0], // p2_walk06
    [213.0,    0.0,  70.0,  94.0], // p2_walk07
    [284.0,    0.0,  70.0,  94.0], // p2_walk08
    [213.0,   95.0,  70.0,  94.0], // p2_walk09
    [355.0,    0.0,  70.0,  94.0], // p2_walk10
    [284.0,   95.0,  70.0,  94.0], // p2_walk11
];

const P3_SHEET: &'static str = "Platformer Art Complete Pack/Base pack/Player/p3_spritesheet.png";

#[rustfmt::skip]
const P3_RECTS: [[f32; 4]; 16] = [
    [365.0,   98.0,  69.0,  71.0], // p3_duck  
    [  0.0,  196.0,  66.0,  92.0], // p3_front 
    [438.0,    0.0,  69.0,  92.0], // p3_hurt  
    [438.0,   93.0,  67.0,  94.0], // p3_jump  
    [ 67.0,  196.0,  66.0,  92.0], // p3_stand 
    [  0.0,    0.0,  72.0,  97.0], // p3_walk01
    [ 73.0,    0.0,  72.0,  97.0], // p3_walk02
    [146.0,    0.0,  72.0,  97.0], // p3_walk03
    [  0.0,   98.0,  72.0,  97.0], // p3_walk04
    [ 73.0,   98.0,  72.0,  97.0], // p3_walk05
    [146.0,   98.0,  72.0,  97.0], // p3_walk06
    [219.0,    0.0,  72.0,  97.0], // p3_walk07
    [292.0,    0.0,  72.0,  97.0], // p3_walk08
    [219.0,   98.0,  72.0,  97.0], // p3_walk09
    [365.0,    0.0,  72.0,  97.0], // p3_walk10
    [292.0,   98.0,  72.0,  97.0], // p3_walk11
];

pub fn setup_players(
    duration: Duration,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
) -> ([Handle<TextureAtlas>; 3], Handle<SpriteSheetAnimation>) {
    let p1_handle = asset_server.load(P1_SHEET);
    let p2_handle = asset_server.load(P2_SHEET);
    let p3_handle = asset_server.load(P3_SHEET);

    let mut p1_sheet = TextureAtlas::new_empty(p1_handle, Vec2::new(508.0, 288.0));
    let mut p2_sheet = TextureAtlas::new_empty(p2_handle, Vec2::new(494.0, 282.0));
    let mut p3_sheet = TextureAtlas::new_empty(p3_handle, Vec2::new(508.0, 288.0));

    for &[x, y, w, h] in P1_RECTS.iter() {
        p1_sheet.add_texture(Rect {
            min: Vec2::new(x, y),
            max: Vec2::new(x + w, y + h),
        });
    }
    for &[x, y, w, h] in P2_RECTS.iter() {
        p2_sheet.add_texture(Rect {
            min: Vec2::new(x, y),
            max: Vec2::new(x + w, y + h),
        });
    }
    for &[x, y, w, h] in P3_RECTS.iter() {
        p3_sheet.add_texture(Rect {
            min: Vec2::new(x, y),
            max: Vec2::new(x + w, y + h),
        });
    }

    let animation_handle = animations.add(SpriteSheetAnimation::from_range(PLAYER_WALK, duration));

    (
        [
            texture_atlases.add(p1_sheet),
            texture_atlases.add(p2_sheet),
            texture_atlases.add(p3_sheet),
        ],
        animation_handle,
    )
}

const BASE_TILES_SHEET: &'static str =
    "Platformer Art Complete Pack/Base pack/Tiles/tiles_spritesheet.png";

pub const BASE_BOX: [f32; 4] = [0.0, 864.0, 70.0, 70.0];
pub const BASE_BOXALT: [f32; 4] = [0.0, 792.0, 70.0, 70.0];
pub const BASE_BOXCOIN: [f32; 4] = [0.0, 720.0, 70.0, 70.0];
pub const BASE_BOXCOINALT: [f32; 4] = [0.0, 576.0, 70.0, 70.0];
pub const BASE_BOXCOINALT_DISABLED: [f32; 4] = [0.0, 504.0, 70.0, 70.0];
pub const BASE_BOXCOIN_DISABLED: [f32; 4] = [0.0, 648.0, 70.0, 70.0];
pub const BASE_BOXEMPTY: [f32; 4] = [0.0, 432.0, 70.0, 70.0];
pub const BASE_BOXEXPLOSIVE: [f32; 4] = [0.0, 360.0, 70.0, 70.0];
pub const BASE_BOXEXPLOSIVEALT: [f32; 4] = [0.0, 216.0, 70.0, 70.0];
pub const BASE_BOXEXPLOSIVE_DISABLE: [f32; 4] = [0.0, 288.0, 70.0, 70.0];
pub const BASE_BOXITEM: [f32; 4] = [0.0, 144.0, 70.0, 70.0];
pub const BASE_BOXITEMALT: [f32; 4] = [0.0, 0.0, 70.0, 70.0];
pub const BASE_BOXITEMALT_DISABLED: [f32; 4] = [432.0, 432.0, 70.0, 70.0];
pub const BASE_BOXITEM_DISABLED: [f32; 4] = [0.0, 72.0, 70.0, 70.0];
pub const BASE_BOXWARNING: [f32; 4] = [72.0, 648.0, 70.0, 70.0];
pub const BASE_BRICKWALL: [f32; 4] = [216.0, 0.0, 70.0, 70.0];
pub const BASE_BRIDGE: [f32; 4] = [216.0, 72.0, 70.0, 70.0];
pub const BASE_BRIDGELOGS: [f32; 4] = [288.0, 720.0, 70.0, 70.0];
pub const BASE_CASTLE: [f32; 4] = [288.0, 792.0, 70.0, 70.0];
pub const BASE_CASTLECENTER: [f32; 4] = [504.0, 288.0, 70.0, 70.0];
pub const BASE_CASTLECENTER_ROUNDED: [f32; 4] = [504.0, 720.0, 70.0, 70.0];
pub const BASE_CASTLECLIFFLEFT: [f32; 4] = [504.0, 792.0, 70.0, 70.0];
pub const BASE_CASTLECLIFFLEFTALT: [f32; 4] = [648.0, 720.0, 70.0, 70.0];
pub const BASE_CASTLECLIFFRIGHT: [f32; 4] = [648.0, 792.0, 70.0, 70.0];
pub const BASE_CASTLECLIFFRIGHTALT: [f32; 4] = [792.0, 288.0, 70.0, 70.0];
pub const BASE_CASTLEHALF: [f32; 4] = [792.0, 360.0, 70.0, 70.0];
pub const BASE_CASTLEHALFLEFT: [f32; 4] = [432.0, 720.0, 70.0, 70.0];
pub const BASE_CASTLEHALFMID: [f32; 4] = [648.0, 648.0, 70.0, 70.0];
pub const BASE_CASTLEHALFRIGHT: [f32; 4] = [792.0, 648.0, 70.0, 70.0];
pub const BASE_CASTLEHILLLEFT: [f32; 4] = [648.0, 576.0, 70.0, 70.0];
pub const BASE_CASTLEHILLLEFT2: [f32; 4] = [792.0, 576.0, 70.0, 70.0];
pub const BASE_CASTLEHILLRIGHT: [f32; 4] = [792.0, 504.0, 70.0, 70.0];
pub const BASE_CASTLEHILLRIGHT2: [f32; 4] = [792.0, 432.0, 70.0, 70.0];
pub const BASE_CASTLELEDGELEFT: [f32; 4] = [856.0, 868.0, 5.0, 22.0];
pub const BASE_CASTLELEDGERIGHT: [f32; 4] = [842.0, 868.0, 5.0, 22.0];
pub const BASE_CASTLELEFT: [f32; 4] = [792.0, 216.0, 70.0, 70.0];
pub const BASE_CASTLEMID: [f32; 4] = [792.0, 144.0, 70.0, 70.0];
pub const BASE_CASTLERIGHT: [f32; 4] = [792.0, 72.0, 70.0, 70.0];
pub const BASE_DIRT: [f32; 4] = [792.0, 0.0, 70.0, 70.0];
pub const BASE_DIRTCENTER: [f32; 4] = [720.0, 864.0, 70.0, 70.0];
pub const BASE_DIRTCENTER_ROUNDED: [f32; 4] = [720.0, 792.0, 70.0, 70.0];
pub const BASE_DIRTCLIFFLEFT: [f32; 4] = [720.0, 720.0, 70.0, 70.0];
pub const BASE_DIRTCLIFFLEFTALT: [f32; 4] = [720.0, 648.0, 70.0, 70.0];
pub const BASE_DIRTCLIFFRIGHT: [f32; 4] = [720.0, 576.0, 70.0, 70.0];
pub const BASE_DIRTCLIFFRIGHTALT: [f32; 4] = [720.0, 504.0, 70.0, 70.0];
pub const BASE_DIRTHALF: [f32; 4] = [720.0, 432.0, 70.0, 70.0];
pub const BASE_DIRTHALFLEFT: [f32; 4] = [720.0, 360.0, 70.0, 70.0];
pub const BASE_DIRTHALFMID: [f32; 4] = [720.0, 288.0, 70.0, 70.0];
pub const BASE_DIRTHALFRIGHT: [f32; 4] = [720.0, 216.0, 70.0, 70.0];
pub const BASE_DIRTHILLLEFT: [f32; 4] = [720.0, 144.0, 70.0, 70.0];
pub const BASE_DIRTHILLLEFT2: [f32; 4] = [720.0, 72.0, 70.0, 70.0];
pub const BASE_DIRTHILLRIGHT: [f32; 4] = [720.0, 0.0, 70.0, 70.0];
pub const BASE_DIRTHILLRIGHT2: [f32; 4] = [648.0, 864.0, 70.0, 70.0];
pub const BASE_DIRTLEDGELEFT: [f32; 4] = [842.0, 892.0, 5.0, 18.0];
pub const BASE_DIRTLEDGERIGHT: [f32; 4] = [842.0, 912.0, 5.0, 18.0];
pub const BASE_DIRTLEFT: [f32; 4] = [504.0, 432.0, 70.0, 70.0];
pub const BASE_DIRTMID: [f32; 4] = [504.0, 360.0, 70.0, 70.0];
pub const BASE_DIRTRIGHT: [f32; 4] = [648.0, 504.0, 70.0, 70.0];
pub const BASE_DOOR_CLOSEDMID: [f32; 4] = [648.0, 432.0, 70.0, 70.0];
pub const BASE_DOOR_CLOSEDTOP: [f32; 4] = [648.0, 360.0, 70.0, 70.0];
pub const BASE_DOOR_OPENMID: [f32; 4] = [648.0, 288.0, 70.0, 70.0];
pub const BASE_DOOR_OPENTOP: [f32; 4] = [648.0, 216.0, 70.0, 70.0];
pub const BASE_FENCE: [f32; 4] = [648.0, 144.0, 70.0, 70.0];
pub const BASE_FENCEBROKEN: [f32; 4] = [648.0, 72.0, 70.0, 70.0];
pub const BASE_GRASS: [f32; 4] = [648.0, 0.0, 70.0, 70.0];
pub const BASE_GRASSCENTER: [f32; 4] = [576.0, 864.0, 70.0, 70.0];
pub const BASE_GRASSCENTER_ROUNDED: [f32; 4] = [576.0, 792.0, 70.0, 70.0];
pub const BASE_GRASSCLIFFLEFT: [f32; 4] = [576.0, 720.0, 70.0, 70.0];
pub const BASE_GRASSCLIFFLEFTALT: [f32; 4] = [576.0, 648.0, 70.0, 70.0];
pub const BASE_GRASSCLIFFRIGHT: [f32; 4] = [576.0, 576.0, 70.0, 70.0];
pub const BASE_GRASSCLIFFRIGHTALT: [f32; 4] = [576.0, 504.0, 70.0, 70.0];
pub const BASE_GRASSHALF: [f32; 4] = [576.0, 432.0, 70.0, 70.0];
pub const BASE_GRASSHALFLEFT: [f32; 4] = [576.0, 360.0, 70.0, 70.0];
pub const BASE_GRASSHALFMID: [f32; 4] = [576.0, 288.0, 70.0, 70.0];
pub const BASE_GRASSHALFRIGHT: [f32; 4] = [576.0, 216.0, 70.0, 70.0];
pub const BASE_GRASSHILLLEFT: [f32; 4] = [576.0, 144.0, 70.0, 70.0];
pub const BASE_GRASSHILLLEFT2: [f32; 4] = [576.0, 72.0, 70.0, 70.0];
pub const BASE_GRASSHILLRIGHT: [f32; 4] = [576.0, 0.0, 70.0, 70.0];
pub const BASE_GRASSHILLRIGHT2: [f32; 4] = [504.0, 864.0, 70.0, 70.0];
pub const BASE_GRASSLEDGELEFT: [f32; 4] = [849.0, 868.0, 5.0, 24.0];
pub const BASE_GRASSLEDGERIGHT: [f32; 4] = [849.0, 894.0, 5.0, 24.0];
pub const BASE_GRASSLEFT: [f32; 4] = [504.0, 648.0, 70.0, 70.0];
pub const BASE_GRASSMID: [f32; 4] = [504.0, 576.0, 70.0, 70.0];
pub const BASE_GRASSRIGHT: [f32; 4] = [504.0, 504.0, 70.0, 70.0];
pub const BASE_HILL_LARGE: [f32; 4] = [842.0, 720.0, 48.0, 146.0];
pub const BASE_HILL_LARGEALT: [f32; 4] = [864.0, 0.0, 48.0, 146.0];
pub const BASE_HILL_SMALL: [f32; 4] = [792.0, 828.0, 48.0, 106.0];
pub const BASE_HILL_SMALLALT: [f32; 4] = [792.0, 720.0, 48.0, 106.0];
pub const BASE_LADDER_MID: [f32; 4] = [504.0, 144.0, 70.0, 70.0];
pub const BASE_LADDER_TOP: [f32; 4] = [504.0, 72.0, 70.0, 70.0];
pub const BASE_LIQUIDLAVA: [f32; 4] = [504.0, 0.0, 70.0, 70.0];
pub const BASE_LIQUIDLAVATOP: [f32; 4] = [432.0, 864.0, 70.0, 70.0];
pub const BASE_LIQUIDLAVATOP_MID: [f32; 4] = [432.0, 792.0, 70.0, 70.0];
pub const BASE_LIQUIDWATER: [f32; 4] = [504.0, 216.0, 70.0, 70.0];
pub const BASE_LIQUIDWATERTOP: [f32; 4] = [432.0, 648.0, 70.0, 70.0];
pub const BASE_LIQUIDWATERTOP_MID: [f32; 4] = [432.0, 576.0, 70.0, 70.0];
pub const BASE_LOCK_BLUE: [f32; 4] = [432.0, 504.0, 70.0, 70.0];
pub const BASE_LOCK_GREEN: [f32; 4] = [72.0, 576.0, 70.0, 70.0];
pub const BASE_LOCK_RED: [f32; 4] = [432.0, 360.0, 70.0, 70.0];
pub const BASE_LOCK_YELLOW: [f32; 4] = [432.0, 288.0, 70.0, 70.0];
pub const BASE_ROCKHILLLEFT: [f32; 4] = [432.0, 216.0, 70.0, 70.0];
pub const BASE_ROCKHILLRIGHT: [f32; 4] = [432.0, 144.0, 70.0, 70.0];
pub const BASE_ROPEATTACHED: [f32; 4] = [432.0, 72.0, 70.0, 70.0];
pub const BASE_ROPEHORIZONTAL: [f32; 4] = [432.0, 0.0, 70.0, 70.0];
pub const BASE_ROPEVERTICAL: [f32; 4] = [360.0, 864.0, 70.0, 70.0];
pub const BASE_SAND: [f32; 4] = [360.0, 792.0, 70.0, 70.0];
pub const BASE_SANDCENTER: [f32; 4] = [576.0, 864.0, 70.0, 70.0];
pub const BASE_SANDCENTER_ROUNDED: [f32; 4] = [576.0, 792.0, 70.0, 70.0];
pub const BASE_SANDCLIFFLEFT: [f32; 4] = [360.0, 720.0, 70.0, 70.0];
pub const BASE_SANDCLIFFLEFTALT: [f32; 4] = [360.0, 648.0, 70.0, 70.0];
pub const BASE_SANDCLIFFRIGHT: [f32; 4] = [360.0, 576.0, 70.0, 70.0];
pub const BASE_SANDCLIFFRIGHTALT: [f32; 4] = [360.0, 504.0, 70.0, 70.0];
pub const BASE_SANDHALF: [f32; 4] = [360.0, 432.0, 70.0, 70.0];
pub const BASE_SANDHALFLEFT: [f32; 4] = [360.0, 360.0, 70.0, 70.0];
pub const BASE_SANDHALFMID: [f32; 4] = [360.0, 288.0, 70.0, 70.0];
pub const BASE_SANDHALFRIGHT: [f32; 4] = [360.0, 216.0, 70.0, 70.0];
pub const BASE_SANDHILLLEFT: [f32; 4] = [360.0, 144.0, 70.0, 70.0];
pub const BASE_SANDHILLLEFT2: [f32; 4] = [360.0, 72.0, 70.0, 70.0];
pub const BASE_SANDHILLRIGHT: [f32; 4] = [360.0, 0.0, 70.0, 70.0];
pub const BASE_SANDHILLRIGHT2: [f32; 4] = [288.0, 864.0, 70.0, 70.0];
pub const BASE_SANDLEDGELEFT: [f32; 4] = [856.0, 892.0, 5.0, 18.0];
pub const BASE_SANDLEDGERIGHT: [f32; 4] = [856.0, 912.0, 5.0, 18.0];
pub const BASE_SANDLEFT: [f32; 4] = [288.0, 648.0, 70.0, 70.0];
pub const BASE_SANDMID: [f32; 4] = [288.0, 576.0, 70.0, 70.0];
pub const BASE_SANDRIGHT: [f32; 4] = [288.0, 504.0, 70.0, 70.0];
pub const BASE_SIGN: [f32; 4] = [288.0, 432.0, 70.0, 70.0];
pub const BASE_SIGNEXIT: [f32; 4] = [288.0, 360.0, 70.0, 70.0];
pub const BASE_SIGNLEFT: [f32; 4] = [288.0, 288.0, 70.0, 70.0];
pub const BASE_SIGNRIGHT: [f32; 4] = [288.0, 216.0, 70.0, 70.0];
pub const BASE_SNOW: [f32; 4] = [288.0, 144.0, 70.0, 70.0];
pub const BASE_SNOWCENTER: [f32; 4] = [720.0, 864.0, 70.0, 70.0];
pub const BASE_SNOWCENTER_ROUNDED: [f32; 4] = [288.0, 72.0, 70.0, 70.0];
pub const BASE_SNOWCLIFFLEFT: [f32; 4] = [288.0, 0.0, 70.0, 70.0];
pub const BASE_SNOWCLIFFLEFTALT: [f32; 4] = [216.0, 864.0, 70.0, 70.0];
pub const BASE_SNOWCLIFFRIGHT: [f32; 4] = [216.0, 792.0, 70.0, 70.0];
pub const BASE_SNOWCLIFFRIGHTALT: [f32; 4] = [216.0, 720.0, 70.0, 70.0];
pub const BASE_SNOWHALF: [f32; 4] = [216.0, 648.0, 70.0, 70.0];
pub const BASE_SNOWHALFLEFT: [f32; 4] = [216.0, 576.0, 70.0, 70.0];
pub const BASE_SNOWHALFMID: [f32; 4] = [216.0, 504.0, 70.0, 70.0];
pub const BASE_SNOWHALFRIGHT: [f32; 4] = [216.0, 432.0, 70.0, 70.0];
pub const BASE_SNOWHILLLEFT: [f32; 4] = [216.0, 360.0, 70.0, 70.0];
pub const BASE_SNOWHILLLEFT2: [f32; 4] = [216.0, 288.0, 70.0, 70.0];
pub const BASE_SNOWHILLRIGHT: [f32; 4] = [216.0, 216.0, 70.0, 70.0];
pub const BASE_SNOWHILLRIGHT2: [f32; 4] = [216.0, 144.0, 70.0, 70.0];
pub const BASE_SNOWLEDGELEFT: [f32; 4] = [863.0, 868.0, 5.0, 18.0];
pub const BASE_SNOWLEDGERIGHT: [f32; 4] = [863.0, 888.0, 5.0, 18.0];
pub const BASE_SNOWLEFT: [f32; 4] = [144.0, 864.0, 70.0, 70.0];
pub const BASE_SNOWMID: [f32; 4] = [144.0, 792.0, 70.0, 70.0];
pub const BASE_SNOWRIGHT: [f32; 4] = [144.0, 720.0, 70.0, 70.0];
pub const BASE_STONE: [f32; 4] = [144.0, 648.0, 70.0, 70.0];
pub const BASE_STONECENTER: [f32; 4] = [144.0, 576.0, 70.0, 70.0];
pub const BASE_STONECENTER_ROUNDED: [f32; 4] = [144.0, 504.0, 70.0, 70.0];
pub const BASE_STONECLIFFLEFT: [f32; 4] = [144.0, 432.0, 70.0, 70.0];
pub const BASE_STONECLIFFLEFTALT: [f32; 4] = [144.0, 360.0, 70.0, 70.0];
pub const BASE_STONECLIFFRIGHT: [f32; 4] = [144.0, 288.0, 70.0, 70.0];
pub const BASE_STONECLIFFRIGHTALT: [f32; 4] = [144.0, 216.0, 70.0, 70.0];
pub const BASE_STONEHALF: [f32; 4] = [144.0, 144.0, 70.0, 70.0];
pub const BASE_STONEHALFLEFT: [f32; 4] = [144.0, 72.0, 70.0, 70.0];
pub const BASE_STONEHALFMID: [f32; 4] = [144.0, 0.0, 70.0, 70.0];
pub const BASE_STONEHALFRIGHT: [f32; 4] = [72.0, 864.0, 70.0, 70.0];
pub const BASE_STONEHILLLEFT2: [f32; 4] = [72.0, 792.0, 70.0, 70.0];
pub const BASE_STONEHILLRIGHT2: [f32; 4] = [72.0, 720.0, 70.0, 70.0];
pub const BASE_STONELEDGELEFT: [f32; 4] = [863.0, 908.0, 5.0, 24.0];
pub const BASE_STONELEDGERIGHT: [f32; 4] = [864.0, 148.0, 5.0, 24.0];
pub const BASE_STONELEFT: [f32; 4] = [72.0, 504.0, 70.0, 70.0];
pub const BASE_STONEMID: [f32; 4] = [72.0, 432.0, 70.0, 70.0];
pub const BASE_STONERIGHT: [f32; 4] = [72.0, 360.0, 70.0, 70.0];
pub const BASE_STONEWALL: [f32; 4] = [72.0, 288.0, 70.0, 70.0];
pub const BASE_TOCHLIT: [f32; 4] = [72.0, 216.0, 70.0, 70.0];
pub const BASE_TOCHLIT2: [f32; 4] = [72.0, 144.0, 70.0, 70.0];
pub const BASE_TORCH: [f32; 4] = [72.0, 72.0, 70.0, 70.0];
pub const BASE_WINDOW: [f32; 4] = [72.0, 0.0, 70.0, 70.0];

const BASE_ITEMS_SHEET: &'static str =
    "Platformer Art Complete Pack/Base pack/Items/items_spritesheet.png";

pub const BASE_BOMB: [f32; 4] = [432.0, 432.0, 70.0, 70.0];
pub const BASE_BOMBFLASH: [f32; 4] = [432.0, 360.0, 70.0, 70.0];
pub const BASE_BUSH: [f32; 4] = [346.0, 144.0, 70.0, 70.0];
pub const BASE_BUTTONBLUE: [f32; 4] = [288.0, 504.0, 70.0, 70.0];
pub const BASE_BUTTONBLUE_PRESSED: [f32; 4] = [419.0, 72.0, 70.0, 70.0];
pub const BASE_BUTTONGREEN: [f32; 4] = [419.0, 0.0, 70.0, 70.0];
pub const BASE_BUTTONGREEN_PRESSED: [f32; 4] = [418.0, 144.0, 70.0, 70.0];
pub const BASE_BUTTONRED: [f32; 4] = [360.0, 504.0, 70.0, 70.0];
pub const BASE_BUTTONRED_PRESSED: [f32; 4] = [360.0, 432.0, 70.0, 70.0];
pub const BASE_BUTTONYELLOW: [f32; 4] = [360.0, 360.0, 70.0, 70.0];
pub const BASE_BUTTONYELLOW_PRESSED: [f32; 4] = [360.0, 288.0, 70.0, 70.0];
pub const BASE_CACTUS: [f32; 4] = [360.0, 216.0, 70.0, 70.0];
pub const BASE_CHAIN: [f32; 4] = [347.0, 72.0, 70.0, 70.0];
pub const BASE_CLOUD1: [f32; 4] = [0.0, 146.0, 128.0, 71.0];
pub const BASE_CLOUD2: [f32; 4] = [0.0, 73.0, 129.0, 71.0];
pub const BASE_CLOUD3: [f32; 4] = [0.0, 0.0, 129.0, 71.0];
pub const BASE_COINBRONZE: [f32; 4] = [288.0, 432.0, 70.0, 70.0];
pub const BASE_COINGOLD: [f32; 4] = [288.0, 360.0, 70.0, 70.0];
pub const BASE_COINSILVER: [f32; 4] = [288.0, 288.0, 70.0, 70.0];
pub const BASE_FIREBALL: [f32; 4] = [0.0, 435.0, 70.0, 70.0];
pub const BASE_FLAGBLUE: [f32; 4] = [275.0, 72.0, 70.0, 70.0];
pub const BASE_FLAGBLUE2: [f32; 4] = [275.0, 0.0, 70.0, 70.0];
pub const BASE_FLAGBLUEHANGING: [f32; 4] = [216.0, 504.0, 70.0, 70.0];
pub const BASE_FLAGGREEN: [f32; 4] = [216.0, 432.0, 70.0, 70.0];
pub const BASE_FLAGGREEN2: [f32; 4] = [216.0, 360.0, 70.0, 70.0];
pub const BASE_FLAGGREENHANGING: [f32; 4] = [216.0, 288.0, 70.0, 70.0];
pub const BASE_FLAGRED: [f32; 4] = [274.0, 144.0, 70.0, 70.0];
pub const BASE_FLAGRED2: [f32; 4] = [216.0, 216.0, 70.0, 70.0];
pub const BASE_FLAGREDHANGING: [f32; 4] = [203.0, 72.0, 70.0, 70.0];
pub const BASE_FLAGYELLOW: [f32; 4] = [203.0, 0.0, 70.0, 70.0];
pub const BASE_FLAGYELLOW2: [f32; 4] = [202.0, 144.0, 70.0, 70.0];
pub const BASE_FLAGYELLOWHANGING: [f32; 4] = [144.0, 434.0, 70.0, 70.0];
pub const BASE_GEMBLUE: [f32; 4] = [144.0, 362.0, 70.0, 70.0];
pub const BASE_GEMGREEN: [f32; 4] = [144.0, 290.0, 70.0, 70.0];
pub const BASE_GEMRED: [f32; 4] = [144.0, 218.0, 70.0, 70.0];
pub const BASE_GEMYELLOW: [f32; 4] = [131.0, 72.0, 70.0, 70.0];
pub const BASE_KEYBLUE: [f32; 4] = [131.0, 0.0, 70.0, 70.0];
pub const BASE_KEYGREEN: [f32; 4] = [130.0, 146.0, 70.0, 70.0];
pub const BASE_KEYRED: [f32; 4] = [72.0, 435.0, 70.0, 70.0];
pub const BASE_KEYYELLOW: [f32; 4] = [72.0, 363.0, 70.0, 70.0];
pub const BASE_MUSHROOMBROWN: [f32; 4] = [72.0, 291.0, 70.0, 70.0];
pub const BASE_MUSHROOMRED: [f32; 4] = [72.0, 219.0, 70.0, 70.0];
pub const BASE_PARTICLEBRICK1A: [f32; 4] = [0.0, 553.0, 19.0, 14.0];
pub const BASE_PARTICLEBRICK1B: [f32; 4] = [0.0, 530.0, 21.0, 21.0];
pub const BASE_PARTICLEBRICK2A: [f32; 4] = [21.0, 553.0, 19.0, 14.0];
pub const BASE_PARTICLEBRICK2B: [f32; 4] = [0.0, 507.0, 21.0, 21.0];
pub const BASE_PLANT: [f32; 4] = [0.0, 363.0, 70.0, 70.0];
pub const BASE_PLANTPURPLE: [f32; 4] = [0.0, 291.0, 70.0, 70.0];
pub const BASE_ROCK: [f32; 4] = [0.0, 219.0, 70.0, 70.0];
pub const BASE_SNOWHILL: [f32; 4] = [288.0, 216.0, 70.0, 70.0];
pub const BASE_SPIKES: [f32; 4] = [347.0, 0.0, 70.0, 70.0];
pub const BASE_SPRINGBOARDDOWN: [f32; 4] = [432.0, 288.0, 70.0, 70.0];
pub const BASE_SPRINGBOARDUP: [f32; 4] = [432.0, 216.0, 70.0, 70.0];
pub const BASE_STAR: [f32; 4] = [504.0, 288.0, 70.0, 70.0];
pub const BASE_SWITCHLEFT: [f32; 4] = [504.0, 216.0, 70.0, 70.0];
pub const BASE_SWITCHMID: [f32; 4] = [491.0, 72.0, 70.0, 70.0];
pub const BASE_SWITCHRIGHT: [f32; 4] = [491.0, 0.0, 70.0, 70.0];
pub const BASE_WEIGHT: [f32; 4] = [490.0, 144.0, 70.0, 70.0];
pub const BASE_WEIGHTCHAINED: [f32; 4] = [432.0, 504.0, 70.0, 70.0];

const BASE_HUD_SHEET: &'static str =
    "Platformer Art Complete Pack/Base pack/HUD/hud_spritesheet.png";

pub const BASE_HUD_0: [f32; 4] = [230.0, 0.0, 30.0, 38.0];
pub const BASE_HUD_1: [f32; 4] = [196.0, 41.0, 26.0, 37.0];
pub const BASE_HUD_2: [f32; 4] = [55.0, 98.0, 32.0, 38.0];
pub const BASE_HUD_3: [f32; 4] = [239.0, 80.0, 28.0, 38.0];
pub const BASE_HUD_4: [f32; 4] = [238.0, 122.0, 29.0, 38.0];
pub const BASE_HUD_5: [f32; 4] = [238.0, 162.0, 28.0, 38.0];
pub const BASE_HUD_6: [f32; 4] = [230.0, 40.0, 30.0, 38.0];
pub const BASE_HUD_7: [f32; 4] = [226.0, 206.0, 32.0, 39.0];
pub const BASE_HUD_8: [f32; 4] = [192.0, 206.0, 32.0, 40.0];
pub const BASE_HUD_9: [f32; 4] = [196.0, 0.0, 32.0, 39.0];
pub const BASE_HUD_COINS: [f32; 4] = [55.0, 0.0, 47.0, 47.0];
pub const BASE_HUD_GEM_BLUE: [f32; 4] = [104.0, 0.0, 46.0, 36.0];
pub const BASE_HUD_GEM_GREEN: [f32; 4] = [98.0, 185.0, 46.0, 36.0];
pub const BASE_HUD_GEM_RED: [f32; 4] = [98.0, 147.0, 46.0, 36.0];
pub const BASE_HUD_GEM_YELLOW: [f32; 4] = [98.0, 223.0, 46.0, 36.0];
pub const BASE_HUD_HEARTEMPTY: [f32; 4] = [0.0, 47.0, 53.0, 45.0];
pub const BASE_HUD_HEARTFULL: [f32; 4] = [0.0, 94.0, 53.0, 45.0];
pub const BASE_HUD_HEARTHALF: [f32; 4] = [0.0, 0.0, 53.0, 45.0];
pub const BASE_HUD_KEYBLUE: [f32; 4] = [146.0, 147.0, 44.0, 40.0];
pub const BASE_HUD_KEYBLUE_DISABLED: [f32; 4] = [150.0, 38.0, 44.0, 40.0];
pub const BASE_HUD_KEYGREEM_DISABLED: [f32; 4] = [104.0, 38.0, 44.0, 40.0];
pub const BASE_HUD_KEYGREEN: [f32; 4] = [192.0, 122.0, 44.0, 40.0];
pub const BASE_HUD_KEYRED: [f32; 4] = [193.0, 80.0, 44.0, 40.0];
pub const BASE_HUD_KEYRED_DISABLED: [f32; 4] = [192.0, 164.0, 44.0, 40.0];
pub const BASE_HUD_KEYYELLOW: [f32; 4] = [146.0, 189.0, 44.0, 40.0];
pub const BASE_HUD_KEYYELLOW_DISABLED: [f32; 4] = [147.0, 80.0, 44.0, 40.0];
pub const BASE_HUD_P1: [f32; 4] = [55.0, 49.0, 47.0, 47.0];
pub const BASE_HUD_P1ALT: [f32; 4] = [0.0, 141.0, 47.0, 47.0];
pub const BASE_HUD_P2: [f32; 4] = [49.0, 141.0, 47.0, 47.0];
pub const BASE_HUD_P2ALT: [f32; 4] = [0.0, 190.0, 47.0, 47.0];
pub const BASE_HUD_P3: [f32; 4] = [49.0, 190.0, 47.0, 47.0];
pub const BASE_HUD_P3ALT: [f32; 4] = [98.0, 98.0, 47.0, 47.0];
pub const BASE_HUD_X: [f32; 4] = [0.0, 239.0, 30.0, 28.0];

const BASE_ENEMIES_SHEET: &'static str =
    "Platformer Art Complete Pack/Base pack/Enemies/enemies_spritesheet.png";

// blockerBody = 203 0 51 51
// blockerMad = 136 66 51 51
// blockerSad = 188 66 51 51
// fishDead = 0 69 66 42
// fishSwim1 = 76 0 66 42
// fishSwim2 = 73 43 62 43
// flyDead = 143 0 59 33
// flyFly1 = 0 32 72 36
// flyFly2 = 0 0 75 31
// pokerMad = 255 0 48 146
// pokerSad = 304 0 48 146
// slimeDead = 0 112 59 12
// slimeWalk1 = 52 125 50 28
// slimeWalk2 = 0 125 51 26
// snailShell = 103 119 44 30
// snailShell_upsidedown = 148 118 44 30
// snailWalk1 = 143 34 54 31
// snailWalk2 = 67 87 57 31
