use bevy::{prelude::*, render::render_resource::TextureUsages};

pub const TILE_SIZE: u32 = 70;
pub const SHEET_W: u16 = 27;
pub const SHEET_H: u16 = 35;
pub struct SpriteAssets {
    pub tile_texture: Handle<TextureAtlas>,
}

pub fn setup_sprites(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = assets.load("tilesheet.png");

    commands.insert_resource(SpriteAssets {
        tile_texture: atlases.add(TextureAtlas::from_grid(
            texture_handle,
            Vec2::splat(TILE_SIZE as f32),
            SHEET_W as usize,
            SHEET_H as usize,
        )),
    });
}

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        if let AssetEvent::Created { handle } = event {
            if let Some(mut texture) = textures.get_mut(handle) {
                texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST;
            }
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
