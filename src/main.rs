use std::time::Duration;

use benimator::{AnimationPlugin, Play, SpriteSheetAnimation};
use bevy::prelude::*;
use bevy_pixel_camera::{PixelBorderPlugin, PixelCameraBundle, PixelCameraPlugin};

mod assets;
use assets::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(PixelCameraPlugin)
        .add_plugin(PixelBorderPlugin {
            color: Color::rgb(0.0, 0.0, 0.0),
        })
        .add_plugin(AnimationPlugin)
        .add_startup_system(setup.system())
        .add_system(keyboard_input_system.system())
        .run();
}

struct Player;

fn keyboard_input_system(
    mut player: Query<(&Player, &mut Transform)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let mut dir = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::Left) {
        dir += Vec2::new(-1.0, 0.0);
    }

    if keyboard_input.pressed(KeyCode::Right) {
        dir += Vec2::new(1.0, 0.0);
    }

    if keyboard_input.pressed(KeyCode::Down) {
        dir += Vec2::new(0.0, -1.0);
    }

    if keyboard_input.pressed(KeyCode::Up) {
        dir += Vec2::new(0.0, 1.0);
    }

    for (_, mut trans) in player.iter_mut() {
        trans.translation += Vec3::new(8.0 * dir.x, 8.0 * dir.y, 0.0);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
    animations: ResMut<Assets<SpriteSheetAnimation>>,
) {
    commands.spawn_bundle(PixelCameraBundle::from_resolution(960, 640));

    let graphics = setup_textures(
        Duration::from_millis(60),
        asset_server,
        texture_atlases,
        animations,
    );

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: graphics.sheet,
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..Default::default()
        })
        .insert(Player)
        .insert(graphics.p1_walk)
        .insert(Play);
}
