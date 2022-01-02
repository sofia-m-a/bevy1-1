#![feature(int_abs_diff)]

use bevy::{
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuLimits},
};

mod assets;
mod camera;
mod map;
use assets::{
    set_texture_filters_to_nearest, setup_sprites, SpriteAssets, SHEET_H, SHEET_W, TILE_SIZE,
};
use camera::*;
use map::map::{chunk_load_unload, Map};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum GameState {
    Splash,
    Level,
}

fn main() {
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
        //.add_plugin(AnimationPlugin)
        //.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
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
                .with_system(chunk_load_unload),
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
    mut player: Query<(&mut CameraCenter, With<Player>)>,
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

    for mut hint in player.iter_mut() {
        let center = hint.0.center;
        let speed = if keyboard_input.pressed(KeyCode::LShift) {
            50.0
        } else {
            15.0
        };
        hint.0.center = center + dir * speed;
    }
}

fn setup(
    mut commands: Commands,
    //mut rapier: ResMut<RapierConfiguration>,
    mut color: ResMut<ClearColor>,
    graphics: Res<SpriteAssets>,
) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(SofiaCamera {
            view: Rect::default(),
            aspect_ratio: ASPECT_X / ASPECT_Y,
        });

    commands
        .spawn()
        .insert(Map::new())
        .insert(Transform::from_scale(
            Vec2::splat(TILE_SIZE as f32).extend(1.0),
        ));
    // physics
    //rapier.scale = TILE_SIZE as f32;
    //rapier.gravity = Vec2::new(0.0, -40.0).into();

    // clear color for sky
    *color = ClearColor(SKY_COLOR);

    // generate_chunk(
    //     &mut commands,
    //     &mut map_query,
    //     graphics.tile_texture.clone(),
    //     0u16,
    // );
}

fn setup_player(mut commands: Commands, graphics: Res<SpriteAssets>) {
    // let player_size = &RECTS[crate::assets::Tile::P1Walk01 as usize][2..=3];
    // let (w, h) = (player_size[0] / TILE_SIZE, player_size[1] / TILE_SIZE);
    // let player_body = RigidBodyBundle {
    //     velocity: RigidBodyVelocity {
    //         angvel: 0.0,
    //         linvel: Vec2::new(0.0, 3.0).into(),
    //     },

    //     mass_properties: RigidBodyMassProps {
    //         flags: RigidBodyMassPropsFlags::ROTATION_LOCKED,
    //         ..Default::default()
    //     },
    //     ..Default::default()
    // };
    // let player_shape = ColliderBundle {
    //     collider_type: ColliderType::Solid,
    //     shape: ColliderShape::cuboid(w / 2.0, h / 2.0),
    //     ..Default::default()
    // };

    // commands
    //     .spawn_bundle(SpriteSheetBundle {
    //         texture_atlas: graphics.texture.clone(),
    //         ..Default::default()
    //     })
    //     .insert(Player)
    //     .insert(graphics.p2_walk.clone())
    //     .insert(Play)
    //     .insert_bundle(player_body)
    //     .insert(RigidBodyPositionSync::Discrete)
    //     .insert_bundle(player_shape);

    commands
        .spawn()
        .insert(CameraCenter { center: Vec2::ZERO })
        .insert(Player);
}
