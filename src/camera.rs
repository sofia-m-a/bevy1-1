use bevy::{prelude::*, render::camera::OrthographicProjection};

pub const ASPECT_X: f32 = 4.0;
pub const ASPECT_Y: f32 = 3.0;

#[derive(Clone, Copy, Component)]
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

#[derive(Component)]
pub struct CameraMarker;

pub fn camera_center(
    hints: Query<&CameraHint>,
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

struct AspectRatio(pub f32);

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
    cameras: Query<
        (&OrthographicProjection, &Transform),
        Or<(Changed<OrthographicProjection>, Changed<Transform>)>,
    >,
    mut borders: Query<(&mut Sprite, &mut Transform, &Border), Without<OrthographicProjection>>,
    aspect: Res<AspectRatio>,
) {
    if let Some((p, t)) = cameras.iter().next() {
        let z = p.far - 0.2; // Just in front of camera
        let width = p.right - p.left;
        let height = p.top - p.bottom;
        let cur_aspect = width / height;
        let (trim_x, trim_y) = if cur_aspect > aspect.0 {
            (width - aspect.0 * height, 0.0)
        } else if cur_aspect < aspect.0 {
            (0.0, height - aspect.0 * width)
        } else {
            (0.0, 0.0)
        };
        let t = t.translation;

        for (mut sprite, mut transform, border) in borders.iter_mut() {
            match *border {
                Border::Left => {
                    *transform = Transform::from_xyz(t.x - width / 2.0, t.y, z);
                    sprite.custom_size = Some(Vec2::new(2.0 * trim_x, height));
                }
                Border::Right => {
                    *transform = Transform::from_xyz(t.x + width / 2.0, t.y, z);
                    sprite.custom_size = Some(Vec2::new(2.0 * trim_x, height));
                }
                Border::Top => {
                    *transform = Transform::from_xyz(t.x, t.y + height / 2.0, z);
                    sprite.custom_size = Some(Vec2::new(width, 2.0 * trim_y));
                }
                Border::Bottom => {
                    *transform = Transform::from_xyz(t.x, t.y - height / 2.0, z);
                    sprite.custom_size = Some(Vec2::new(width, 2.0 * trim_y));
                }
            }
        }
    }
}

pub struct LetterboxPlugin;

impl Plugin for LetterboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(letterbox_init)
            .add_system_to_stage(CoreStage::PostUpdate, letterbox)
            .insert_resource(AspectRatio(ASPECT_X / ASPECT_Y));
    }
}
