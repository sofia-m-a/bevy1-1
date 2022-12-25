use bevy::prelude::*;

use crate::animation::{Animation, AnimationAsset};

pub const TILE_SIZE: u32 = 70;
pub const SHEET_W: u16 = 28;
pub const SHEET_H: u16 = 59;
pub const PIXEL_MODEL_TRANSFORM: Transform = Transform::from_scale(Vec3::new(
    1.0 / TILE_SIZE as f32,
    1.0 / TILE_SIZE as f32,
    1.0,
));

#[derive(Resource)]
pub struct SpriteAssets {
    pub tile_texture: Handle<Image>,
    pub player_atlas: Handle<TextureAtlas>,
    pub p1_walk_animation: Animation,
    pub p2_walk_animation: Animation,
    pub p3_walk_animation: Animation,
    pub kenney_pixel_font: Handle<Font>,
    pub text_style: TextStyle,
    pub blank_texture: Handle<Image>,
}

pub fn setup_sprites(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut animations: ResMut<Assets<AnimationAsset>>,
) {
    let texture_handle = assets.load("numbering.png");

    let p1_handle: Handle<Image> =
        assets.load("Platformer Art Complete Pack/Base pack/Player/p1_spritesheet.png");
    let p2_handle: Handle<Image> =
        assets.load("Platformer Art Complete Pack/Base pack/Player/p2_spritesheet.png");
    let p3_handle: Handle<Image> =
        assets.load("Platformer Art Complete Pack/Base pack/Player/p3_spritesheet.png");

    let mut player_atlas = TextureAtlas::new_empty(p1_handle, Vec2::new(508.0, 288.0));

    for &[x, y, w, h] in [
        P1_DUCK, P1_FRONT, P1_HURT, P1_JUMP, P1_STAND, P1_WALK01, P1_WALK02, P1_WALK03, P1_WALK04,
        P1_WALK05, P1_WALK06, P1_WALK07, P1_WALK08, P1_WALK09, P1_WALK10, P1_WALK11, P2_DUCK,
        P2_FRONT, P2_HURT, P2_JUMP, P2_STAND, P2_WALK01, P2_WALK02, P2_WALK03, P2_WALK04,
        P2_WALK05, P2_WALK06, P2_WALK07, P2_WALK08, P2_WALK09, P2_WALK10, P2_WALK11, P3_DUCK,
        P3_FRONT, P3_HURT, P3_JUMP, P3_STAND, P3_WALK01, P3_WALK02, P3_WALK03, P3_WALK04,
        P3_WALK05, P3_WALK06, P3_WALK07, P3_WALK08, P3_WALK09, P3_WALK10, P3_WALK11,
    ]
    .iter()
    {
        player_atlas.add_texture(bevy::math::Rect {
            min: Vec2::new(x as f32, y as f32),
            max: Vec2::new((x + w) as f32, (y + h) as f32),
        });
    }

    let p1_walk_animation = Animation(animations.add(AnimationAsset(
        benimator::Animation::from_indices(5..=15, benimator::FrameRate::from_fps(20.0)),
    )));
    let p2_walk_animation = Animation(animations.add(AnimationAsset(
        benimator::Animation::from_indices(5..=15, benimator::FrameRate::from_fps(20.0)),
    )));
    let p3_walk_animation = Animation(animations.add(AnimationAsset(
        benimator::Animation::from_indices(5..=15, benimator::FrameRate::from_fps(20.0)),
    )));

    let kenney_pixel_font = assets.load("kenney_fontpackage/Fonts/Kenney Pixel.ttf");
    let text_style = TextStyle {
        font: kenney_pixel_font.clone(),
        font_size: 60.0,
        color: Color::WHITE,
    };

    let blank_texture = assets.load("1x1.png");

    commands.insert_resource(SpriteAssets {
        tile_texture: texture_handle,
        player_atlas: atlases.add(player_atlas),
        p1_walk_animation,
        p2_walk_animation,
        p3_walk_animation,
        kenney_pixel_font,
        text_style,
        blank_texture,
    });
}

pub const P1_DUCK: [u32; 4] = [365, 98, 69, 71];
pub const P1_FRONT: [u32; 4] = [0, 196, 66, 92];
pub const P1_HURT: [u32; 4] = [438, 0, 69, 92];
pub const P1_JUMP: [u32; 4] = [438, 93, 67, 94];
pub const P1_STAND: [u32; 4] = [67, 196, 66, 92];
pub const P1_WALK01: [u32; 4] = [0, 0, 72, 97];
pub const P1_WALK02: [u32; 4] = [73, 0, 72, 97];
pub const P1_WALK03: [u32; 4] = [146, 0, 72, 97];
pub const P1_WALK04: [u32; 4] = [0, 98, 72, 97];
pub const P1_WALK05: [u32; 4] = [73, 98, 72, 97];
pub const P1_WALK06: [u32; 4] = [146, 98, 72, 97];
pub const P1_WALK07: [u32; 4] = [219, 0, 72, 97];
pub const P1_WALK08: [u32; 4] = [292, 0, 72, 97];
pub const P1_WALK09: [u32; 4] = [219, 98, 72, 97];
pub const P1_WALK10: [u32; 4] = [365, 0, 72, 97];
pub const P1_WALK11: [u32; 4] = [292, 98, 72, 97];

pub const P2_DUCK: [u32; 4] = [355, 95, 67, 72];
pub const P2_FRONT: [u32; 4] = [0, 190, 66, 92];
pub const P2_HURT: [u32; 4] = [426, 0, 67, 92];
pub const P2_JUMP: [u32; 4] = [423, 95, 66, 94];
pub const P2_STAND: [u32; 4] = [67, 190, 66, 92];
pub const P2_WALK01: [u32; 4] = [0, 0, 70, 94];
pub const P2_WALK02: [u32; 4] = [71, 0, 70, 94];
pub const P2_WALK03: [u32; 4] = [142, 0, 70, 94];
pub const P2_WALK04: [u32; 4] = [0, 95, 70, 94];
pub const P2_WALK05: [u32; 4] = [71, 95, 70, 94];
pub const P2_WALK06: [u32; 4] = [142, 95, 70, 94];
pub const P2_WALK07: [u32; 4] = [213, 0, 70, 94];
pub const P2_WALK08: [u32; 4] = [284, 0, 70, 94];
pub const P2_WALK09: [u32; 4] = [213, 95, 70, 94];
pub const P2_WALK10: [u32; 4] = [355, 0, 70, 94];
pub const P2_WALK11: [u32; 4] = [284, 95, 70, 94];

pub const P3_DUCK: [u32; 4] = [365, 98, 69, 71];
pub const P3_FRONT: [u32; 4] = [0, 196, 66, 92];
pub const P3_HURT: [u32; 4] = [438, 0, 69, 92];
pub const P3_JUMP: [u32; 4] = [438, 93, 67, 94];
pub const P3_STAND: [u32; 4] = [67, 196, 66, 92];
pub const P3_WALK01: [u32; 4] = [0, 0, 72, 97];
pub const P3_WALK02: [u32; 4] = [73, 0, 72, 97];
pub const P3_WALK03: [u32; 4] = [146, 0, 72, 97];
pub const P3_WALK04: [u32; 4] = [0, 98, 72, 97];
pub const P3_WALK05: [u32; 4] = [73, 98, 72, 97];
pub const P3_WALK06: [u32; 4] = [146, 98, 72, 97];
pub const P3_WALK07: [u32; 4] = [219, 0, 72, 97];
pub const P3_WALK08: [u32; 4] = [292, 0, 72, 97];
pub const P3_WALK09: [u32; 4] = [219, 98, 72, 97];
pub const P3_WALK10: [u32; 4] = [365, 0, 72, 97];
pub const P3_WALK11: [u32; 4] = [292, 98, 72, 97];
