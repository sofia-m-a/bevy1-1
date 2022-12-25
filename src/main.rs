#![feature(trivial_bounds)]
#![feature(let_chains)]
#![feature(exclusive_range_pattern)]

// use benimator::*;
use bevy::{
    prelude::*,
    render::{render_resource::WgpuLimits, settings::WgpuSettings},
    transform::TransformSystem,
};
use bevy_ecs_tilemap::{prelude::TilemapRenderSettings, TilemapPlugin};
use bevy_rapier2d::prelude::*;
use bevy_tweening::TweeningPlugin;
use iyes_loopless::prelude::*;

mod animation;
mod assets;
mod camera;
mod helpers;
mod world;
use animation::{AnimationAsset, AnimationPlugin, AnimationState};
use assets::{setup_sprites, SpriteAssets, P1_WALK01, PIXEL_MODEL_TRANSFORM, TILE_SIZE};
use camera::*;
use world::{
    add_level_resource,
    brushes::Gen,
    player::{keyboard_input_system, setup_camera, setup_player, control_switch_input_system},
    LevelResource,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum GameState {
    Splash,
    Level,
}

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
        .insert_resource(TilemapRenderSettings {
            render_chunk_size: UVec2::new(500, 32),
        })
        .add_plugin(TilemapPlugin)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            bevy::render::camera::camera_system::<LetterboxProjection>,
        )
        .add_plugin(LetterboxBorderPlugin {
            color: Color::rgb(0.1, 0.1, 0.1),
        })
        .add_system_to_stage(
            CoreStage::PostUpdate,
            update_sofia_camera.after(TransformSystem::TransformPropagate),
        )
        .add_plugin(TweeningPlugin)
        .add_plugin(AnimationPlugin)
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
                .with_system(setup_camera)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Level)
                .with_system(world::chunk_loader)
                .with_system(control_switch_input_system)
                .with_system(keyboard_input_system)
                .into(),
        )
        .run();
}

const SKY_COLOR: Color = Color::rgb_linear(0.2, 0.6, 1.0);

fn update_clear_colour(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::Level));
}

fn setup(
    mut _commands: Commands,
    mut rapier: ResMut<RapierConfiguration>,
    mut color: ResMut<ClearColor>,
) {
    // physics
    rapier.gravity = Vec2::new(0.0, 0.0).into();

    // clear color for sky
    *color = ClearColor(SKY_COLOR);
}
