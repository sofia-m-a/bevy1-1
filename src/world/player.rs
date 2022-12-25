use bevy::prelude::*;

use crate::camera::{spawn_borders, BorderColor, CameraGuide, LetterboxCameraBundle, SofiaCamera};

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
pub struct FreeCam;

#[derive(Clone, Copy, Debug, Component)]
pub struct CameraBox;

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

fn spawn_camera_box(commands: &mut Commands, freecam: Entity) {
    let b = commands
        .spawn(CameraBox)
        .insert(SpatialBundle::VISIBLE_IDENTITY)
        .id();
    spawn_box(commands, b, 20.0, 12.0);
    commands.entity(freecam).add_child(b);
}

pub fn keyboard_input_system(
    mut camera: Query<&mut Transform, With<FreeCam>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
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
