use bevy::{prelude::*, render::camera::OrthographicProjection};

pub const ASPECT_X: f32 = 4.0;
pub const ASPECT_Y: f32 = 3.0;

#[derive(Clone, Copy)]
pub enum CameraHint {
    Show(Vec2),
    NoShow(Vec2),
}

pub struct CameraCenter(pub Vec2);
pub struct CameraMarker;

pub fn camera_center(
    hints: Query<&CameraHint>,
    center: Res<CameraCenter>,
    windows: Res<Windows>,
    mut cam: Query<(&mut Transform, &mut OrthographicProjection), With<CameraMarker>>,
) {
    let mut max_manhattan = f32::INFINITY;
    let mut min_manhattan = 0.0;
    for &h in hints.iter() {
        match h {
            CameraHint::Show(v) => {
                let v = v - center.0;
                min_manhattan = f32::max(f32::max(v.x.abs(), v.y.abs()), min_manhattan);
            }
            CameraHint::NoShow(v) => {
                let v = v - center.0;
                max_manhattan = f32::min(f32::min(v.x.abs(), v.y.abs()), max_manhattan)
            }
        }
    }

    let size = f32::min(max_manhattan, min_manhattan);

    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width(), window.height());

    let sx = window_size.x / ASPECT_X;
    let sy = window_size.y / ASPECT_Y;
    let scale = f32::min(sx, sy);

    for (mut trans, mut proj) in cam.iter_mut() {
        if sx >= sy {
            let slack = window_size.x - scale * ASPECT_X;
            proj.left = slack / 2.0;
            proj.right = window_size.x - slack / 2.0;
            proj.top = window_size.y;
            proj.bottom = 0.0;
        } else {
            let slack = window_size.y - scale * ASPECT_Y;
            proj.top = window_size.y - slack / 2.0;
            proj.bottom = slack / 2.0;
            proj.right = window_size.x;
            proj.left = 0.0;
        }

        proj.scale = 0.7;
        trans.translation = Vec3::new(center.0.x, center.0.y, 0.0);
    }
}
