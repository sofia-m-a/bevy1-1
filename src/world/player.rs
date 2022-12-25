use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier2d::prelude::*;

use super::LevelResource;
use crate::{
    assets::{SpriteAssets, P1_WALK01, PIXEL_MODEL_TRANSFORM, TILE_SIZE},
    camera::{spawn_borders, BorderColor, CameraGuide, LetterboxCameraBundle, SofiaCamera},
    AnimationState,
};

fn arrows_to_vec(keyboard_input: Res<Input<KeyCode>>) -> Vec2 {
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

    dir
}

#[derive(Clone, Copy, Debug, Component)]
pub struct KeyboardController;

#[derive(Clone, Copy, Debug, Component)]
pub struct FreeCam;

#[derive(Clone, Copy, Debug, Component)]
pub struct Player;

#[derive(Clone, Copy, Debug, Component)]
pub struct CameraBox;

#[derive(Clone, Copy, Debug, Component)]
pub struct PlayerBox;

pub fn control_switch_input_system(
    mut commands: Commands,
    freecam: Query<(Entity, Option<&KeyboardController>), With<FreeCam>>,
    player: Query<(Entity, Option<&KeyboardController>), With<Player>>,
    mut player_box: Query<&mut Visibility, (With<PlayerBox>, Without<CameraBox>)>,
    mut camera_box: Query<&mut Visibility, (Without<PlayerBox>, With<CameraBox>)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::C) {
        if let (Some((fc, fck)), Some((pl, plk))) = (freecam.iter().next(), player.iter().next()) {
            dbg!(fc, fck, pl, plk);
            if fck.is_some() {
                commands.entity(fc).remove::<KeyboardController>();
                commands.entity(pl).insert(KeyboardController);

                for mut vis in player_box.iter_mut() {
                    *vis = Visibility::VISIBLE;
                }

                for mut vis in camera_box.iter_mut() {
                    *vis = Visibility::INVISIBLE;
                }
            }

            if plk.is_some() {
                commands.entity(pl).remove::<KeyboardController>();
                commands.entity(fc).insert(KeyboardController);

                for mut vis in player_box.iter_mut() {
                    *vis = Visibility::INVISIBLE;
                }

                for mut vis in camera_box.iter_mut() {
                    *vis = Visibility::VISIBLE;
                }
            }
        }
    }
}

fn spawn_box(commands: &mut Commands, entity: Entity, w: f32, h: f32) {
    commands.entity(entity).with_children(|parent| {
        parent
            .spawn(CameraGuide::Center)
            .insert(SpatialBundle::from_transform(Transform::from_translation(
                Vec3::new(0.0, 0.0, 0.0),
            )))
            .log_components();
        parent
            .spawn(CameraGuide::MustBeOnscreen)
            .insert(SpatialBundle::from_transform(Transform::from_translation(
                Vec3::new(-w, -h, 0.0),
            )));
        parent
            .spawn(CameraGuide::MustBeOnscreen)
            .insert(SpatialBundle::from_transform(Transform::from_translation(
                Vec3::new(-w, h, 0.0),
            )));
        parent
            .spawn(CameraGuide::MustBeOnscreen)
            .insert(SpatialBundle::from_transform(Transform::from_translation(
                Vec3::new(w, -h, 0.0),
            )));
        parent
            .spawn(CameraGuide::MustBeOnscreen)
            .insert(SpatialBundle::from_transform(Transform::from_translation(
                Vec3::new(w, h, 0.0),
            )));
    });
}

fn spawn_player_box(commands: &mut Commands, player: Entity) {
    let b = commands
        .spawn(PlayerBox)
        .insert(SpatialBundle::default())
        .id();
    spawn_box(commands, b, 12.0, 8.0);
    commands.entity(player).add_child(b);
}

fn spawn_camera_box(commands: &mut Commands, freecam: Entity) {
    let b = commands
        .spawn(CameraBox)
        .insert(SpatialBundle::INVISIBLE_IDENTITY)
        .id();
    spawn_box(commands, b, 20.0, 12.0);
    commands.entity(freecam).add_child(b);
}

pub fn keyboard_input_system(
    mut player: Query<&mut ExternalImpulse, (With<Player>, With<KeyboardController>)>,
    mut camera: Query<&mut Transform, (With<FreeCam>, With<KeyboardController>)>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for mut p in player.iter_mut() {
        p.impulse =
            5.0 * arrows_to_vec(Res::clone(&keyboard_input)) * Vec2::new(time.delta_seconds(), 0.0)
    }

    for mut cam in camera.iter_mut() {
        cam.translation += 100.0 *
            (arrows_to_vec(Res::clone(&keyboard_input)) * time.delta_seconds()).extend(0.0);

        if keyboard_input.pressed(KeyCode::Z) {
            cam.scale += (Vec2::new(0.05, 0.05) * time.delta_seconds()).extend(0.0)
        }
        if keyboard_input.pressed(KeyCode::X) {
            cam.scale -= (Vec2::new(0.05, 0.05) * time.delta_seconds()).extend(0.0)
        }
    }
}

pub fn setup_camera(mut commands: Commands, border: Res<BorderColor>) {
    let camera = commands
        .spawn(LetterboxCameraBundle::default())
        .insert(SofiaCamera::new(Transform::default()))
        .id();
    spawn_borders(&mut commands, camera, border);

    let cam = commands
        .spawn(FreeCam)
        .insert(SpatialBundle::default())
        .id();
    spawn_camera_box(&mut commands, cam);
}

pub fn setup_player(
    mut commands: Commands,
    level: Res<LevelResource>,
    graphics: Res<SpriteAssets>,
) {
    let player_size = [P1_WALK01[2], P1_WALK01[3]];
    let (w, h) = (
        player_size[0] as f32 / TILE_SIZE as f32,
        player_size[1] as f32 / TILE_SIZE as f32,
    );

    let player_model = commands
        .spawn(SpriteSheetBundle {
            texture_atlas: graphics.player_atlas.clone(),
            ..Default::default()
        })
        .insert(PIXEL_MODEL_TRANSFORM)
        .insert(VisibilityBundle::default())
        .insert(graphics.p1_walk_animation.clone())
        .insert(AnimationState::default())
        .id();

    let player = commands
        .spawn(Player)
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(w * 0.5, h * 0.5))
        .insert(Ccd::enabled())
        .insert(Sleeping::disabled())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(ExternalImpulse::default())
        .insert(SpatialBundle::default())
        .insert(KeyboardController)
        .add_child(player_model)
        .id();
    spawn_player_box(&mut commands, player);
    commands.entity(level.0).add_child(player);
}
