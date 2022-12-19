#![feature(trivial_bounds)]
// use benimator::*;
use bevy::{
    prelude::*,
    render::{camera::ScalingMode, render_resource::WgpuLimits, settings::WgpuSettings},
};
use bevy_ecs_tilemap::TilemapPlugin;
// use bevy_pixel_camera::{PixelBorderPlugin, PixelCameraPlugin, PixelCameraBundle};
use crate::map::brushes;
use bevy_rapier2d::prelude::*;
use extent::Extent;
use iyes_loopless::prelude::*;
use rand_pcg::Pcg64;

use assets::{
    set_texture_filters_to_nearest, setup_sprites, Animation, AnimationAsset, SpriteAssets,
    P1_WALK01, TILE_SIZE,
};
mod assets;
mod camera;
mod map;
use camera::*;
use map::{
    add_level_resource, brushes::Gen, chunk_loader, level_graph::debug_graph, LevelResource,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum GameState {
    Splash,
    Level,
}

// Create the player component
#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(pub benimator::State);

fn main() {
    //debug_graph();
    App::new()
        .insert_resource(WgpuSettings {
            limits: WgpuLimits {
                max_texture_array_layers: 2048,
                ..Default::default()
            },
            ..Default::default()
        })
        .add_loopless_state(GameState::Splash)
        .add_plugins(DefaultPlugins.set(bevy::render::texture::ImagePlugin::default_nearest()))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: true,
            enabled: true,
            ..Default::default()
        })
        //.add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        //.add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            bevy::render::camera::camera_system::<LetterboxProjection>,
        )
        .add_plugin(LetterboxBorderPlugin {
            color: Color::rgb(0.1, 0.1, 0.1),
        })
        .add_system(animate)
        .add_asset::<AnimationAsset>()
        .add_startup_system(add_level_resource)
        .insert_resource(ClearColor(SKY_COLOR))
        .add_enter_system(GameState::Splash, setup_sprites)
        .add_system(update_clear_colour.run_in_state(GameState::Splash))
        .init_resource::<Gen>()
        .add_enter_system_set(
            GameState::Level,
            ConditionSet::new()
                .with_system(setup)
                .with_system(setup_player)
                .with_system(map::load_level)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Level)
                .with_system(keyboard_input_system)
                .into(), //.with_system(chunk_loader),
        )
        .run();
}

fn animate(
    time: Res<Time>,
    animations: Res<Assets<AnimationAsset>>,
    mut query: Query<(&mut AnimationState, &mut TextureAtlasSprite, &Animation)>,
) {
    for (mut player, mut texture, animation) in query.iter_mut() {
        // Update the state
        if let Some(a) = animations.get(&animation.0) {
            player.update(a, time.delta());
        }

        // Update the texture atlas
        texture.index = player.frame_index();
    }
}

const SKY_COLOR: Color = Color::rgb_linear(0.2, 0.6, 1.0);

fn update_clear_colour(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::Level));
}

#[derive(Component)]
struct Player;

fn keyboard_input_system(
    mut player: Query<(&mut ExternalImpulse, With<Player>)>,
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

    if let Some(mut p) = player.iter_mut().next() {
        p.0.impulse = (100.0 * dir).into();
    }
}

fn setup(
    mut commands: Commands,
    mut rapier: ResMut<RapierConfiguration>,
    mut color: ResMut<ClearColor>,
) {
    commands
        .spawn(LetterboxCameraBundle::default())
        .insert(SofiaCamera::default());

    // physics
    rapier.gravity = Vec2::new(0.0, 0.0).into();

    // clear color for sky
    *color = ClearColor(SKY_COLOR);
}

fn setup_player(mut commands: Commands, level: Res<LevelResource>, graphics: Res<SpriteAssets>) {
    let player_size = [P1_WALK01[2], P1_WALK01[3]];
    let (w, h) = (
        player_size[0] as f32 / TILE_SIZE as f32,
        player_size[1] as f32 / TILE_SIZE as f32,
    );

    let player = commands
        .spawn(Player)
        .insert(CameraCenter)
        .insert(RigidBody::Dynamic)
        //.insert(Collider::cuboid(w * 0.5, h * 0.5))
        .insert(Ccd::enabled())
        .insert(Sleeping::disabled())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(ExternalImpulse::default())
        .insert(graphics.p1_walk_animation.clone())
        .insert(AnimationState::default())
        .insert(SpriteSheetBundle {
            texture_atlas: graphics.player_atlas.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(Onscreen)
                .insert(Transform::from_translation(Vec3::new(-20.0, -10.0, 0.0)));
            parent
                .spawn(Onscreen)
                .insert(Transform::from_translation(Vec3::new(20.0, 10.0, 0.0)));
        })
        .id();
    commands.entity(level.0).add_child(player);
}
