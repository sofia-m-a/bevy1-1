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
use iyes_loopless::prelude::*;

mod assets;
mod camera;
mod helpers;
mod world;
use assets::{setup_sprites};
use camera::*;
use world::{
    add_level_resource,
    brushes::Gen,
    player::{keyboard_input_system, setup_camera},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct GameStateLevel;

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
        .add_loopless_state(GameStateLevel)
        .add_plugins(DefaultPlugins.set(bevy::render::texture::ImagePlugin::default_nearest()))
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
        .add_startup_system(add_level_resource)
        .insert_resource(ClearColor(SKY_COLOR))
        .add_enter_system(GameStateLevel, setup_sprites)
        .init_resource::<Gen>()
        .add_enter_system_set(
            GameStateLevel,
            ConditionSet::new()
                .with_system(setup)
                .with_system(setup_camera)
                .into(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameStateLevel)
                .with_system(world::chunk_loader)
                .with_system(keyboard_input_system)
                .into(),
        )
        .run();
}

const SKY_COLOR: Color = Color::rgb_linear(0.2, 0.6, 1.0);

fn setup(
    mut _commands: Commands,
    mut color: ResMut<ClearColor>,
) {
    // clear color for sky
    *color = ClearColor(SKY_COLOR);
}
