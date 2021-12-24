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
