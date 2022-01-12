#![feature(int_abs_diff)]
#![feature(bool_to_option)]

use assets::{
    set_texture_filters_to_nearest, setup_sprites, SpriteAssets, P1_WALK01, SHEET_H, SHEET_W,
    TILE_SIZE,
};
use benimator::{AnimationPlugin, Play};
use bevy::{
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuLimits},
};
use heron::prelude::*;

mod assets;
mod camera;
mod map;
use camera::*;
use map::{chunk_loader, level_graph::debug_graph, Gen};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum GameState {
    Splash,
    Level,
}

fn main() {
    //debug_graph();

    App::new()
        .insert_resource(WgpuOptions {
            limits: WgpuLimits {
                max_texture_array_layers: 2048,
                ..Default::default()
            },
            ..Default::default()
        })
        .add_state(GameState::Splash)
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_plugin(LetterboxCameraPlugin)
        .add_system_set(SystemSet::on_enter(GameState::Splash).with_system(setup_sprites))
        .add_system_set(SystemSet::on_update(GameState::Splash).with_system(update_clear_colour))
        .add_system_set(
            SystemSet::on_enter(GameState::Level)
                .with_system(setup)
                .with_system(setup_player),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Level)
                .with_system(keyboard_input_system)
                .with_system(chunk_loader),
        )
        .add_system(set_texture_filters_to_nearest)
        .run();
}
const SKY_COLOR: Color = Color::rgb_linear(0.2, 0.6, 1.0);

fn update_clear_colour(mut app_state: ResMut<State<GameState>>) {
    app_state.set(GameState::Level).unwrap()
}

#[derive(Component)]
struct Player;

fn keyboard_input_system(
    mut commands: Commands,
    mut player: Query<(&mut RigidBodyVelocityComponent, With<Player>)>,
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
        p.0 .0.linvel = (20.0 * dir).into();
    }
}

fn setup(
    mut commands: Commands,
    mut rapier: ResMut<RapierConfiguration>,
    mut color: ResMut<ClearColor>,
) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(SofiaCamera {
            view: Rect::default(),
            aspect_ratio: ASPECT_X / ASPECT_Y,
        });

    commands.insert_resource(Gen::new());

    // physics
    rapier.scale = TILE_SIZE as f32;
    rapier.gravity = Vec2::new(0.0, -12.0).into();

    // clear color for sky
    *color = ClearColor(SKY_COLOR);
}

fn setup_player(mut commands: Commands, graphics: Res<SpriteAssets>) {
    let player_size = [P1_WALK01[2], P1_WALK01[3]];
    let (w, h) = (player_size[0] / TILE_SIZE, player_size[1] / TILE_SIZE);
    let player_body = RigidBodyBundle {
        position: RigidBodyPositionComponent(RigidBodyPosition {
            position: Vec2::new(0.0, 10.0).into(),
            ..Default::default()
        }),
        velocity: RigidBodyVelocityComponent(RigidBodyVelocity {
            angvel: 0.0,
            linvel: Vec2::new(0.0, 3.0).into(),
        }),
        mass_properties: RigidBodyMassPropsComponent(RigidBodyMassProps {
            flags: RigidBodyMassPropsFlags::ROTATION_LOCKED,
            ..Default::default()
        }),
        ..Default::default()
    };
    let player_shape = ColliderBundle {
        collider_type: ColliderTypeComponent(ColliderType::Solid),
        shape: ColliderShapeComponent(ColliderShape::cuboid(w as f32 / 2.0, h as f32 / 2.0)),
        ..Default::default()
    };

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: graphics.p1_texture.clone(),
            ..Default::default()
        })
        .insert(graphics.walk_animation.clone())
        .insert(Play)
        .insert_bundle(player_shape)
        .insert_bundle(player_body)
        .insert(RigidBodyPositionSync::Discrete)
        .insert(CameraCenter)
        .insert(Player);
}
