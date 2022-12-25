use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::{
        camera::{CameraProjection, CameraRenderGraph},
        primitives::Frustum,
        view::VisibleEntities,
    },
    sprite::Anchor,
};
use bevy_tweening::Lerp;

pub const ASPECT_X: u32 = 16;
pub const ASPECT_Y: u32 = 9;
pub const ASPECT: f32 = ASPECT_X as f32 / ASPECT_Y as f32;

#[derive(Clone, Copy, Debug, Component)]
pub enum CameraGuide {
    Attractor { attraction_radius: f32 },
    Center,
    MustBeOnscreen,
}

pub fn get_camera_rect(camera_transform: &Transform, proj: &LetterboxProjection) -> Rect {
    Rect::from_corners(
        camera_transform
            .transform_point(Vec3::new(-1.0, -1.0 / proj.desired_aspect_ratio, 0.0))
            .xy(),
        camera_transform
            .transform_point(Vec3::new(1.0, 1.0 / proj.desired_aspect_ratio, 0.0))
            .xy(),
    )
}

#[derive(Debug, Component)]
pub struct SofiaCamera {
    pub target_transform: Transform,
}

impl SofiaCamera {
    pub fn new(t: Transform) -> Self {
        Self {
            target_transform: t,
        }
    }
}

struct TransLerp(Transform);

impl Lerp for TransLerp {
    type Scalar = f32;
    fn lerp(&self, other: &TransLerp, t: &f32) -> Self {
        TransLerp(Transform {
            translation: self.0.translation.lerp(other.0.translation, *t),
            rotation: self.0.rotation.slerp(other.0.rotation, *t),
            scale: self.0.scale.lerp(other.0.scale, *t),
        })
    }
}

pub fn update_sofia_camera(
    time: Res<Time>,
    guides: Query<(&CameraGuide, &GlobalTransform, &ComputedVisibility), Without<SofiaCamera>>,
    mut cams: Query<(&mut Transform, &LetterboxProjection, &mut SofiaCamera)>,
) {
    for (mut cam_trans, proj, mut cam) in cams.iter_mut() {
        let aspect = proj.desired_aspect_ratio;

        let mut center_sum = Vec2::ZERO;
        let mut center_n = 0.0;
        let mut lowest = None;
        let mut highest = None;

        for (&guide, &transform, vis) in guides.iter() {
            if vis.is_visible_in_hierarchy() {
                let transform = transform.compute_transform();
                match guide {
                    CameraGuide::Attractor { attraction_radius } => {
                        let delta = transform.translation.xy() - cam_trans.translation.xy();
                        let r_2 = delta.length_squared();
                        let a_2 = attraction_radius * attraction_radius;
                        let distance_inside_2 = f32::max(0.0, (a_2 - r_2) / a_2);
                        center_sum += delta * distance_inside_2.powi(2);
                        center_n += 1.0;
                    }
                    CameraGuide::Center => {
                        center_sum += transform.translation.xy();
                        center_n += 1.0;
                        lowest = Some(
                            lowest
                                .unwrap_or(transform.translation.xy())
                                .min(transform.translation.xy()),
                        );
                        highest = Some(
                            highest
                                .unwrap_or(transform.translation.xy())
                                .max(transform.translation.xy()),
                        );
                    }
                    CameraGuide::MustBeOnscreen => {
                        lowest = Some(
                            lowest
                                .unwrap_or(transform.translation.xy())
                                .min(transform.translation.xy()),
                        );
                        highest = Some(
                            highest
                                .unwrap_or(transform.translation.xy())
                                .max(transform.translation.xy()),
                        );
                    }
                }
            }
        }

        let center = if center_n > 0.0 {
            center_sum / center_n
        } else {
            warn!("No centers for camera to follow");
            Vec2::ZERO
        };

        let size = match (lowest, highest) {
            (Some(vl), Some(vh)) => Vec2::max(
                (vl - cam.target_transform.translation.xy()).abs(),
                (vh - cam.target_transform.translation.xy()).abs(),
            ),
            (Some(v), None) | (None, Some(v)) => v - cam.target_transform.translation.xy(),
            (None, None) => Vec2::new(1.0, 1.0),
        };

        let size = Vec2::new(size.x, size.y / aspect);
        let size = f32::max(size.x, size.y);

        let old_transform = cam.target_transform;
        cam.target_transform = Transform {
            translation: center.extend(cam_trans.translation.z),
            rotation: cam_trans.rotation,
            scale: Vec2::new(size, size).extend(cam_trans.scale.z),
        };

        let dt = 5.0 * time.delta_seconds();
        let a = TransLerp(*cam_trans).lerp(&TransLerp(old_transform), &dt);
        let b = TransLerp(old_transform).lerp(&TransLerp(cam.target_transform), &dt);
        *cam_trans = a.lerp(&b, &dt).0;
    }
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

// Component
#[derive(Component)]
enum Border {
    Left,
    Right,
    Top,
    Bottom,
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
    pub visibility: VisibilityBundle,
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
            visibility: VisibilityBundle::default(),
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
            .add_system_to_stage(CoreStage::PostUpdate, resize_borders);
    }
}

/// Resource used to specify the color of the opaque border.
#[derive(Clone, Debug, Resource)]
pub struct BorderColor(Color);

/// Function to spawn the opaque border.
pub fn spawn_borders(commands: &mut Commands, camera: Entity, color: Res<BorderColor>) {
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
        .entity(camera)
        .add_child(left)
        .add_child(right)
        .add_child(top)
        .add_child(bottom);
}

fn resize_borders(
    cameras: Query<(&LetterboxProjection, &Children), Changed<LetterboxProjection>>,
    mut borders: Query<(&mut Sprite, &mut Transform, &Border), Without<LetterboxProjection>>,
) {
    if let Ok((projection, children)) = cameras.get_single() {
        let alpha = 1.0 / projection.desired_aspect_ratio;

        for &child in children.iter() {
            if let Ok((mut sprite, mut transform, border)) = borders.get_mut(child) {
                let vec_hor = Vec2::new(projection.fraction_x, 2.0 * alpha);
                let vec_ver = Vec2::new(2.0, projection.fraction_y * alpha);
                match border {
                    Border::Left => {
                        let trans = Vec3::new(-1.0 - projection.fraction_x, -1.0 * alpha, -0.1);
                        *transform = Transform::from_translation(trans);
                        sprite.custom_size = Some(vec_hor);
                    }
                    Border::Right => {
                        let trans = Vec3::new(1.0, -1.0 * alpha, -0.1);
                        *transform = Transform::from_translation(trans);
                        sprite.custom_size = Some(vec_hor);
                    }
                    Border::Top => {
                        let trans = Vec3::new(-1.0, (-1.0 - projection.fraction_y) * alpha, -0.1);
                        *transform = Transform::from_translation(trans);
                        sprite.custom_size = Some(vec_ver);
                    }
                    Border::Bottom => {
                        let trans = Vec3::new(-1.0, 1.0 * alpha, -0.1);
                        *transform = Transform::from_translation(trans);
                        sprite.custom_size = Some(vec_ver);
                    }
                }
            }
        }
    }
}
