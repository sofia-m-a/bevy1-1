use std::time::Duration;

use bevy::{prelude::*, render::camera::OrthographicProjection};
use bevy_easings::*;

pub const ASPECT_X: f32 = 4.0;
pub const ASPECT_Y: f32 = 3.0;

#[derive(Clone, Copy)]
pub enum CameraHint {
    Attract {
        center: Vec2,
        radius_snap: f32,
        radius_attract: f32,
    },
    Center {
        center: Vec2,
    },
}

pub struct CameraMarker;

pub fn camera_center(
    mut commands: Commands,
    hints: Query<&CameraHint>,
    windows: Res<Windows>,
    mut cam: Query<(&mut Transform, &mut OrthographicProjection), With<CameraMarker>>,
) {
    let mut center_mean = Vec2::ZERO;
    let mut n = 0;

    for &hint in hints.iter() {
        match hint {
            CameraHint::Center { center } => {
                center_mean += center;
                n += 1
            }
            CameraHint::Attract { .. } => (),
        }
    }

    let center_mean = center_mean / (n as f32);
    let mut target = center_mean;
    let mut n = 1;

    for &hint in hints.iter() {
        match hint {
            CameraHint::Center { .. } => (),
            CameraHint::Attract {
                center,
                radius_attract,
                radius_snap,
            } => {
                let distance2 = (center - center_mean).length_squared();
                let attraction = f32::min(0.0, distance2 - radius_attract * radius_attract);
                let snap = f32::min(0.0, distance2 - radius_snap * radius_snap);
                let weight = attraction + snap * snap * snap;
                target += weight * center;
                n += 1;
            }
        }
    }

    let mut target = Transform::from_translation((target / (n as f32)).extend(0.0));
    target.scale /= 1.2;

    for mut cam in cam.iter_mut() {
        *cam.0 = target;
    }
}
