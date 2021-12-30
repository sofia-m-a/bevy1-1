use bevy::{prelude::*, render::texture::FilterMode};

pub const TILE_SIZE: u32 = 70;
pub const SHEET_W: u16 = 27;
pub const SHEET_H: u16 = 35;
pub struct SpriteAssets {
    pub tile_texture: Handle<TextureAtlas>,
}

pub fn setup_sprites(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = assets.load("tilesheet.png");
    let atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::splat(TILE_SIZE as f32),
        SHEET_W as usize,
        SHEET_H as usize,
    );
    let handle = texture_atlases.add(atlas);

    commands.insert_resource(SpriteAssets {
        tile_texture: handle,
    });
}

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Texture>>,
    mut textures: ResMut<Assets<Texture>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    texture.sampler.min_filter = FilterMode::Nearest;
                }
            }
            _ => (),
        }
    }
}

// fn make_animation(frames: &[Tile], duration: Duration) -> SpriteSheetAnimation {
//     SpriteSheetAnimation::from_frames(
//         frames
//             .iter()
//             .map(|t| Frame {
//                 duration,
//                 index: *t as u32,
//             })
//             .collect(),
//     )
// }
