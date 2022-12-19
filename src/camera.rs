use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::{
        camera::{CameraProjection, CameraRenderGraph, OrthographicProjection, ScalingMode},
        primitives::Frustum,
        view::VisibleEntities,
    },
    sprite::Anchor,
};

use crate::assets::TILE_SIZE;

pub const ASPECT_X: u32 = 16;
pub const ASPECT_Y: u32 = 9;
pub const ASPECT: f32 = ASPECT_X as f32 / ASPECT_Y as f32;

#[derive(Clone, Copy, Component)]
pub struct AttractCamera {
    pub radius_snap: f32,
    pub radius_attract: f32,
}

#[derive(Clone, Copy, Component)]
pub struct CameraCenter;

#[derive(Clone, Copy, Component)]
pub struct Onscreen;

#[derive(Debug, Default, Component)]
pub struct SofiaCamera {
    pub view: Rect,
}

pub fn camera_center(
    centers: Query<&Transform, (With<CameraCenter>, Without<Camera>)>,
    attractors: Query<(&AttractCamera, &Transform), Without<Camera>>,
    onscreen: Query<&Transform, (With<Onscreen>, Without<Camera>)>,
    mut cam: Query<
        (
            &mut Transform,
            &mut OrthographicProjection,
            &mut SofiaCamera,
        ),
        With<Camera>,
    >,
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
    let new_projection = if size.x > size.y {
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
        cam.2.view = Rect::new(
            new_projection.left,
            new_projection.bottom,
            new_projection.right,
            new_projection.top,
        );
    }

    // let target = Transform::from_translation((camera_center).extend(def_camera_plane));

    // for mut cam in cam.iter_mut() {
    //     *cam.0 = target;
    // }
}

#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct LetterboxProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,

    pub desired_aspect_ratio: f32,
    pub fraction_x: f32,
    pub fraction_y: f32,
}

impl CameraProjection for LetterboxProjection {
    fn get_projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            // NOTE: near and far are swapped to invert the depth range from [0,1] to [1,0]
            // This is for interoperability with pipelines using infinite reverse perspective projections.
            self.far,
            self.near,
        )
    }

    fn update(&mut self, width: f32, height: f32) {
        let (actual_width, actual_height) = if width > height * self.desired_aspect_ratio {
            (height * self.desired_aspect_ratio, height)
        } else {
            (width, width / self.desired_aspect_ratio)
        };

        self.fraction_x = width / actual_width - 1.0;
        self.fraction_y = height / actual_height - 1.0;

        self.left = -1.0 - self.fraction_x;
        self.right = 1.0 + self.fraction_x;
        self.bottom = (-1.0 - self.fraction_y) / self.desired_aspect_ratio;
        self.top = (1.0 + self.fraction_y) / self.desired_aspect_ratio;
    }

    fn far(&self) -> f32 {
        self.far
    }
}

impl Default for LetterboxProjection {
    fn default() -> Self {
        Self {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
            near: 0.0,
            far: 1000.0,
            desired_aspect_ratio: ASPECT,
            fraction_x: 0.0,
            fraction_y: 0.0,
        }
    }
}

#[derive(Bundle)]
pub struct LetterboxCameraBundle {
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub letterbox_projection: LetterboxProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera_2d: Camera2d,
}

impl Default for LetterboxCameraBundle {
    fn default() -> Self {
        let letterbox_projection = LetterboxProjection::default();
        let far = letterbox_projection.far - 0.1;
        let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
        let view_projection =
            letterbox_projection.get_projection_matrix() * transform.compute_matrix().inverse();
        let frustum = Frustum::from_view_projection(
            &view_projection,
            &transform.translation,
            &transform.back(),
            letterbox_projection.far(),
        );
        Self {
            camera: Camera::default(),
            camera_render_graph: CameraRenderGraph::new(bevy::core_pipeline::core_2d::graph::NAME),
            letterbox_projection,
            visible_entities: VisibleEntities::default(),
            frustum,
            transform,
            global_transform: Default::default(),
            camera_2d: Default::default(),
        }
    }
}

/// Provides an opaque border around the desired resolution.
pub struct LetterboxBorderPlugin {
    pub color: Color,
}

impl Plugin for LetterboxBorderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BorderColor(self.color))
            .add_startup_system(spawn_borders)
            .add_system_to_stage(CoreStage::PostUpdate, resize_borders);
    }
}

/// Resource used to specify the color of the opaque border.
#[derive(Clone, Debug, Resource)]
pub struct BorderColor(Color);

// Component
#[derive(Component)]
enum Border {
    Left,
    Right,
    Top,
    Bottom,
}

/// System to spawn the opaque border. Automatically added by the plugin as a
/// startup system.
pub fn spawn_borders(mut commands: Commands, color: Res<BorderColor>) {
    let mut spawn_border = |name: &'static str, side: Border| -> Entity {
        commands
            .spawn((
                Name::new(name),
                side,
                SpriteBundle {
                    sprite: Sprite {
                        anchor: Anchor::BottomLeft,
                        color: color.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ))
            .id()
    };

    let left = spawn_border("Left", Border::Left);
    let right = spawn_border("Right", Border::Right);
    let top = spawn_border("Top", Border::Top);
    let bottom = spawn_border("Bottom", Border::Bottom);

    commands
        .spawn((SpatialBundle::default(), Name::new("Borders")))
        .push_children(&[left, right, top, bottom]);
}

fn resize_borders(
    cameras: Query<
        (&LetterboxProjection, &GlobalTransform),
        Or<(Changed<LetterboxProjection>, Changed<GlobalTransform>)>,
    >,
    mut borders: Query<(&mut Sprite, &mut Transform, &Border), Without<LetterboxProjection>>,
) {
    if let Ok((projection, transform)) = cameras.get_single() {
        let z = projection.far - 0.2;
        let alpha = 1.0 / projection.desired_aspect_ratio;
        for (mut sprite, mut transform, border) in borders.iter_mut() {
            match border {
                Border::Left => {
                    *transform = Transform::from_xyz(-1.0 - projection.fraction_x, -1.0 * alpha, z);
                    sprite.custom_size = Some(Vec2::new(projection.fraction_x, 2.0 * alpha));
                }
                Border::Right => {
                    *transform = Transform::from_xyz(1.0, -1.0 * alpha, z);
                    sprite.custom_size = Some(Vec2::new(projection.fraction_x, 2.0 * alpha));
                }
                Border::Top => {
                    *transform =
                        Transform::from_xyz(-1.0, (-1.0 - projection.fraction_y) * alpha, z);
                    sprite.custom_size = Some(Vec2::new(2.0, projection.fraction_y * alpha));
                }
                Border::Bottom => {
                    *transform = Transform::from_xyz(-1.0, 1.0 * alpha, z);
                    sprite.custom_size = Some(Vec2::new(2.0, projection.fraction_y * alpha));
                }
            }
        }

        // let width = projection.trim_x / 2.0;
        // let height = projection.trim_y * alpha / 2.0;
        // let left = transform.translation().x + match projection.window_origin {
        //     WindowOrigin::Center => -1.0,
        //     WindowOrigin::BottomLeft => 0.0,
        // };
        // let right = transform.translation().x + match projection.window_origin {
        //     WindowOrigin::Center => 1.0,
        //     WindowOrigin::BottomLeft => 1.0,
        // };
        // let bottom = transform.translation().y + match projection.window_origin {
        //     WindowOrigin::Center => -1.0 * alpha,
        //     WindowOrigin::BottomLeft => 0.0,
        // };
        // let top = transform.translation().y + match projection.window_origin {
        //     WindowOrigin::Center => 1.0 * alpha,
        //     WindowOrigin::BottomLeft => 1.0 * alpha,
        // };
        // dbg!(left, right, top, bottom, width, height, transform.translation(), projection.trim_x, projection.trim_y);
    }
}
