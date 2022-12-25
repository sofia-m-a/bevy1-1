use bevy::prelude::*;

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
    pub kenney_pixel_font: Handle<Font>,
    pub text_style: TextStyle,
    pub blank_texture: Handle<Image>,
}

pub fn setup_sprites(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = assets.load("numbering.png");

    let kenney_pixel_font = assets.load("kenney_fontpackage/Fonts/Kenney Pixel.ttf");
    let text_style = TextStyle {
        font: kenney_pixel_font.clone(),
        font_size: 60.0,
        color: Color::WHITE,
    };

    let blank_texture = assets.load("1x1.png");

    commands.insert_resource(SpriteAssets {
        tile_texture: texture_handle,
        kenney_pixel_font,
        text_style,
        blank_texture,
    });
}
