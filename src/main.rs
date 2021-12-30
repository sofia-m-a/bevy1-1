use benimator::{AnimationPlugin, Play};
use bevy::prelude::*;
use bevy_loading::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_tilemap::prelude::v0::*;

mod assets;
mod brushes;
mod camera;
mod gen;
mod grid;
use assets::{
    set_texture_filters_to_nearest, setup_sprites, SpriteAssets, SHEET_H, SHEET_W, TILE_SIZE,
};
use brushes::{GroundSet, GroundTileType};
use camera::*;
use gen::generate_island;
use grid::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum GameState {
    Splash,
    Level,
}

fn main() {
    App::build()
        .add_state(GameState::Splash)
        .add_plugins(DefaultPlugins)
        //.add_system(keyboard_input_system.system())
        .add_plugin(AnimationPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(LoadingPlugin {
            loading_state: GameState::Splash,
            next_state: GameState::Level,
        })
        .add_plugin(TilemapPlugin)
        .add_system_set(SystemSet::on_enter(GameState::Splash).with_system(setup_sprites.system()))
        .add_system_set(
            SystemSet::on_update(GameState::Splash).with_system(update_clear_colour.system()),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Level)
                .with_system(setup.system())
                .with_system(setup_player.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Level)
                .with_system(keyboard_input_system.system())
                .with_system(camera_center.system()),
        )
        .add_system(set_texture_filters_to_nearest.system())
        .run();
}
const SKY_COLOR: Color = Color::rgb_linear(0.2, 0.6, 1.0);

fn update_clear_colour(mut color: ResMut<ClearColor>, counter: Res<bevy_loading::ProgressCounter>) {
    *color = ClearColor(SKY_COLOR * f32::from(counter.progress()));
}

struct Player;

fn keyboard_input_system(
    mut player: Query<(&mut CameraHint, With<Player>)>,
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
        if let CameraHint::Center { center } = *hint.0 {
            *hint.0 = CameraHint::Center {
                center: center + dir * 15.0,
            };
        }
    }
}

fn setup(
    mut commands: Commands,
    mut rapier: ResMut<RapierConfiguration>,
    mut color: ResMut<ClearColor>,
    graphics: Res<SpriteAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(CameraMarker);

    // physics
    rapier.scale = TILE_SIZE as f32;
    rapier.gravity = Vec2::new(0.0, -40.0).into();

    // clear color for sky
    *color = ClearColor(SKY_COLOR);

    // let mut tilemap = Tilemap::new(
    //     graphics.tile_texture.clone(),
    //     (SHEET_W as u32) * TILE_SIZE,
    //     (SHEET_H as u32) * TILE_SIZE,
    // );
    let mut tilemap = TilemapBuilder::new()
        .texture_atlas(graphics.tile_texture.clone())
        .texture_dimensions(TILE_SIZE, TILE_SIZE)
        .auto_chunk()
        .add_layer(
            TilemapLayer {
                kind: LayerKind::Dense, // scenery
            },
            0,
        )
        .add_layer(
            TilemapLayer {
                kind: LayerKind::Sparse, // main tiles
            },
            1,
        )
        .add_layer(
            TilemapLayer {
                kind: LayerKind::Sparse, // special tiles
            },
            2,
        )
        .z_layers(3)
        .finish()
        .unwrap();

    generate_island(&mut tilemap).unwrap();

    let tilemap_components = TilemapBundle {
        tilemap,
        visible: Visible {
            is_visible: true,
            is_transparent: true,
        },
        transform: Default::default(),
        global_transform: Default::default(),
    };
    commands.spawn_bundle(tilemap_components);
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
        .insert(CameraHint::Center { center: Vec2::ZERO })
        .insert(Player);
}
