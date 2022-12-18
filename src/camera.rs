use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::camera::{CameraProjection, OrthographicProjection, ScalingMode, Viewport},
};

use crate::assets::TILE_SIZE;

pub const ASPECT_X: f32 = 16.0;
pub const ASPECT_Y: f32 = 9.0;
pub const ASPECT: f32 = ASPECT_Y / ASPECT_X;

#[derive(Clone, Copy, Component)]
pub struct AttractCamera {
    pub radius_snap: f32,
    pub radius_attract: f32,
}

#[derive(Clone, Copy, Component)]
pub struct CameraCenter;


#[derive(Clone, Copy, Component)]
pub struct Onscreen;

#[derive(Debug, Component)]
pub struct SofiaCamera {
    pub view: Rect,
}

pub fn camera_center(
    centers: Query<&Transform, (With<CameraCenter>, Without<Camera>)>,
    attractors: Query<(&AttractCamera, &Transform), Without<Camera>>,
    onscreen: Query<&Transform, (With<Onscreen>, Without<Camera>)>,
    mut cam: Query<(&mut Transform, &mut OrthographicProjection, &mut SofiaCamera), With<Camera>>,
) {
    let center_mean = centers.iter().fold((Vec2::ZERO, 0.0), |(c, n), v| {
        (c + v.translation.xy(), n + 1.0)
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

    let mut lo = Vec2::ZERO;
    let mut hi = Vec2::ZERO;
    for t in centers.iter().chain(onscreen.iter()) {
        lo = lo.min(t.translation.xy());
        hi = hi.max(t.translation.xy());
    }

    let size = Vec2::max((camera_center - lo).abs(), (camera_center - hi).abs());

    let scale = TILE_SIZE as f32;
    let def_camera_plane = OrthographicProjection::default().far - 0.1;
    let new_projection =
        if size.x > size.y {
            OrthographicProjection {
                scale,
                scaling_mode: ScalingMode::None,
                // left: camera_center.x - size.x / 2.0,
                // right: camera_center.x + size.x / 2.0,
                // top: camera_center.y + (size.x / ASPECT) / 2.0,
                // bottom: camera_center.y - (size.x / ASPECT) / 2.0,
                ..Default::default()
            }
        } else {
            OrthographicProjection {
                scale,
                scaling_mode: ScalingMode::None,
                // left: camera_center.x - (size.y * ASPECT) / 2.0,
                // right: camera_center.x + (size.y * ASPECT) / 2.0,
                // top: camera_center.y + size.y / 2.0,
                // bottom: camera_center.y - size.y / 2.0,
                ..Default::default()
            }
        };
    
    let new_translation = Transform::from_translation(camera_center.extend(def_camera_plane));

    for mut cam in cam.iter_mut() {
        *cam.0 = new_translation;
        *cam.1 = new_projection.clone();
        cam.2.view = Rect::new(new_projection.left, new_projection.bottom, new_projection.right, new_projection.top);
    }

    // let target = Transform::from_translation((camera_center).extend(def_camera_plane));

    // for mut cam in cam.iter_mut() {
    //     *cam.0 = target;
    // }
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
            .spawn(SpriteBundle {
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
    mut cameras: Query<(&mut Camera, &Transform), (With<SofiaCamera>, Without<Border>)>,
    mut borders: Query<(&mut Sprite, &mut Transform, &Border)>,
) {
    if let Some((mut p, t)) = cameras.iter_mut().next() {
        if let Some(rect) = p.logical_viewport_size() {
            let z = 999.9;
            let width = rect.x;
            let height = rect.y;
            let cur_aspect = width / height;
            let (trim_x, trim_y) = if cur_aspect > ASPECT {
                (width - ASPECT * height, 0.0)
            } else if cur_aspect < ASPECT {
                (0.0, height - width / ASPECT)
            } else {
                (0.0, 0.0)
            };
            let t = t.translation;

            debug!("{:?} {:?} {:?} {:?} {:?}", rect, width, height, trim_x, trim_y);
            p.viewport = Some(Viewport {
                physical_position: UVec2::new((trim_x / 2.0).floor() as u32, (trim_y / 2.0).floor() as u32),
                physical_size: UVec2::new((width - trim_x) as u32, (height - trim_y) as u32),
                ..Default::default()
            });

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
        }
    }
}

pub struct LetterboxCameraPlugin;

impl Plugin for LetterboxCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(letterbox_init)
            .add_system_to_stage(CoreStage::PostUpdate, camera_center)
            .add_system_to_stage(CoreStage::PostUpdate, letterbox);
    }
}
