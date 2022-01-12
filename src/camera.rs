use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::camera::{CameraProjection, OrthographicProjection},
};

use crate::assets::TILE_SIZE;

pub const ASPECT_X: f32 = 4.0;
pub const ASPECT_Y: f32 = 3.0;

#[derive(Clone, Copy, Component)]
pub struct AttractCamera {
    pub radius_snap: f32,
    pub radius_attract: f32,
}

#[derive(Clone, Copy, Component)]
pub struct CameraCenter;

#[derive(Component)]
pub struct SofiaCamera {
    pub view: Rect<f32>,
    pub aspect_ratio: f32,
}

pub fn camera_center(
    centers: Query<(&CameraCenter, &Transform), Without<Camera>>,
    attractors: Query<(&AttractCamera, &Transform), Without<Camera>>,
    mut cam: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    let center_mean = centers.iter().fold((Vec2::ZERO, 0.0), |(c, n), v| {
        (c + v.1.translation.xy(), n + 1.0)
    });
    let center_mean = center_mean.0 / center_mean.1;

    let camera_center = attractors
        .iter()
        .map(|a| {
            let distance2 = (a.1.translation.xy() - center_mean).length_squared();
            let attraction = f32::max(0.0, distance2 - a.0.radius_attract * a.0.radius_attract);
            let snap = f32::max(0.0, distance2 - a.0.radius_snap * a.0.radius_snap);
            let weight = -(attraction + snap * snap * snap);
            weight * a.1.translation.xy()
        })
        .fold((Vec2::ZERO, 1.0), |(c, n), v| (c + v, n + 1.0));
    let camera_center = (camera_center.0 + center_mean) / camera_center.1;

    let def_camera_plane = OrthographicProjection::default().far - 0.1;
    let target = Transform::from_translation((camera_center).extend(def_camera_plane));

    for mut cam in cam.iter_mut() {
        *cam.0 = target;
    }
}

#[derive(Clone, Copy, Component)]
enum Border {
    Top,
    Left,
    Right,
    Bottom,
}

fn letterbox_init(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let white_texture = images.add(Image::default());

    use Border::*;
    for &b in [Top, Left, Right, Bottom].iter() {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    ..Default::default()
                },
                texture: white_texture.clone(),
                ..Default::default()
            })
            .insert(b);
    }
}

fn letterbox(
    mut cameras: Query<(&OrthographicProjection, &Transform, &mut SofiaCamera), Without<Border>>,
    mut borders: Query<(&mut Sprite, &mut Transform, &Border)>,
) {
    if let Some((p, t, mut c)) = cameras.iter_mut().next() {
        let z = p.far() - 0.2;
        let width = p.right - p.left;
        let height = p.top - p.bottom;
        let cur_aspect = width / height;
        let (trim_x, trim_y) = if cur_aspect > c.aspect_ratio {
            (width - c.aspect_ratio * height, 0.0)
        } else if cur_aspect < c.aspect_ratio {
            (0.0, height - width / c.aspect_ratio)
        } else {
            (0.0, 0.0)
        };
        let t = t.translation;

        for (mut sprite, mut transform, border) in borders.iter_mut() {
            match *border {
                Border::Left => {
                    *transform = Transform::from_xyz(t.x - width / 2.0, t.y, z);
                    sprite.custom_size = Some(Vec2::new(trim_x, 2.0 * height));
                }
                Border::Right => {
                    *transform = Transform::from_xyz(t.x + width / 2.0, t.y, z);
                    sprite.custom_size = Some(Vec2::new(trim_x, 2.0 * height));
                }
                Border::Top => {
                    *transform = Transform::from_xyz(t.x, t.y + height / 2.0, z);
                    sprite.custom_size = Some(Vec2::new(2.0 * width, trim_y));
                }
                Border::Bottom => {
                    *transform = Transform::from_xyz(t.x, t.y - height / 2.0, z);
                    sprite.custom_size = Some(Vec2::new(2.0 * width, trim_y));
                }
            }
        }

        c.view = Rect {
            left: (t.x - width / 2.0 + trim_x / 2.0) / (TILE_SIZE as f32),
            right: (t.x + width / 2.0 - trim_x / 2.0) / (TILE_SIZE as f32),
            top: (t.y + height / 2.0 - trim_y / 2.0) / (TILE_SIZE as f32),
            bottom: (t.y - height / 2.0 + trim_y / 2.0) / (TILE_SIZE as f32),
        };
    }
}

pub struct LetterboxCameraPlugin;

impl Plugin for LetterboxCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(letterbox_init)
            .add_system_to_stage(CoreStage::Update, camera_center)
            .add_system_to_stage(CoreStage::PostUpdate, letterbox);
    }
}
