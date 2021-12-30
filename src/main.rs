use benimator::{AnimationPlugin, Play};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_loading::prelude::*;
use bevy_rapier2d::prelude::*;

mod assets;
mod brushes;
mod camera;
mod chunk;
mod gen;
mod grid;
use assets::{
    set_texture_filters_to_nearest, setup_sprites, SpriteAssets, SHEET_H, SHEET_W, TILE_SIZE,
};
use brushes::{GroundSet, GroundTileType};
use camera::*;
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
    mut map_query: MapQuery,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(CameraMarker);

    // physics
    rapier.scale = TILE_SIZE;
    rapier.gravity = Vec2::new(0.0, -40.0).into();

    // clear color for sky
    *color = ClearColor(SKY_COLOR);

    //let chunk = Chunk::random_chunk();
    //chunk.load(&mut commands, &graphics.texture, Vec2::new(-8.0, -4.0));

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let material_handle = materials.add(ColorMaterial::texture(graphics.tile_texture.clone()));

    // Creates a new layer builder with a layer entity.
    let (mut layer_builder, _) = LayerBuilder::new(
        &mut commands,
        LayerSettings::new(
            UVec2::new(2, 2),
            UVec2::new(32, 32),
            Vec2::splat(TILE_SIZE),
            Vec2::new(SHEET_W as f32 * TILE_SIZE, SHEET_H as f32 * TILE_SIZE),
        ),
        0u16,
        0u16,
    );

    layer_builder.set_all(TileBundle {
        tile: Tile {
            texture_index: u16::from(brushes::Tile::Ground(
                GroundTileType::LeftCave,
                GroundSet::Cake,
            )),
            ..Default::default()
        },
        ..Default::default()
    });

    // Builds the layer.
    // Note: Once this is called you can no longer edit the layer until a hard sync in bevy.
    let layer_entity = map_query.build_layer(&mut commands, layer_builder, material_handle);

    // Required to keep track of layers for a map internally.
    map.add_layer(&mut commands, 0u16, layer_entity);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-128.0, -128.0, 0.0))
        .insert(GlobalTransform::default());
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
