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
use interpolation::Ease;

pub const ASPECT_X: u32 = 16;
pub const ASPECT_Y: u32 = 9;
pub const ASPECT: f32 = ASPECT_X as f32 / ASPECT_Y as f32;

#[derive(Clone, Copy, Debug, Component)]
pub enum CameraGuide {
    Attractor {
        attraction_radius: f32,
    },
    Center,
    MustBeOnscreen,
}

pub fn get_camera_rect(camera_transform: &Transform, proj: &LetterboxProjection) -> Rect {
    Rect::from_corners(
        camera_transform.transform_point(Vec3::new(-1.0, -1.0 / proj.desired_aspect_ratio, 0.0)).xy(),
        camera_transform.transform_point(Vec3::new(1.0, 1.0 / proj.desired_aspect_ratio, 0.0)).xy(),)
}

#[derive(Debug, Component)]
pub struct SofiaCamera {
    //pub center_s: FirstOrderSmoother2,
    //pub size_s: FirstOrderSmoother2,
    pub snap: f32,
    pub target_center: Vec2,
    pub target_size: Vec2,
}

// #[derive(Clone, Copy, Debug, Component)]
// pub struct FirstOrderSmoother2 {
//     // units per second
//     pub velocity: Vec2,
//     // units per second per second
//     pub max_acceleration: f32,
//     pub target_position: Vec2,
// }

// impl FirstOrderSmoother2 {
//     pub fn from_max_accel(a: f32) -> Self {
//         Self {
//             velocity: Default::default(),
//             max_acceleration: a,
//             target_position: Default::default(),
//         }
//     }
// }
 
// impl FirstOrderSmoother2 {
//     pub fn update(&mut self, transform: Vec2, dt: f32) -> Vec2 {
//         let predicted = transform + self.velocity * dt;
//         let dx = self.target_position - predicted;
//         let target_dv = dx / dt;
//         let dv = target_dv - self.velocity;
//         let a = dv.length() / dt;
//         let a = f32::min(1.0, a);
//         self.velocity += dv.normalize_or_zero() * a;
//         self.velocity * dt
//         // let dx = self.target_position - transform;
//         // let target_v = dx / dt;
//         // let new = target_v - self.velocity;
//         // //dbg!(*self, transform, dx, target_v, new);
//         // self.velocity += new * f32::min(1.0, dt * self.max_acceleration / new.length());
//         // //dbg!(self.velocity);
//         // self.velocity * dt
//     }
// }

// #[cfg(test)]
// mod tests {
//     use bevy::prelude::Vec2;

//     use super::FirstOrderSmoother2;

//     #[test]
//     fn exploration() {
//         let mut smoother = FirstOrderSmoother2::from_max_accel(1.0);
//         let mut position = Vec2::new(0.0, 0.0);
//         smoother.target_position = Vec2::new(10.0, 10.0);
//         for i in 0..100 {
//             position = smoother.update(position, 0.1);
//             println!("{}", position);
//         }
//     }
// }

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
    guides: Query<(&CameraGuide, &GlobalTransform), Without<SofiaCamera>>,
    mut cams: Query<(&mut Transform, &LetterboxProjection, &mut SofiaCamera)>) {

    for (mut cam_trans, proj, mut cam) in cams.iter_mut() {

        let aspect = proj.desired_aspect_ratio;

        let mut center_sum = Vec2::ZERO;
        let mut center_n = 0.0;
        let mut lowest = None;
        let mut highest = None;

        for (&guide, &transform) in guides.iter() {
            let transform = transform.compute_transform();
            match guide {
                CameraGuide::Attractor { attraction_radius } => {
                    let delta = transform.translation.xy() - cam_trans.translation.xy();
                    let r_2 = delta.length_squared();
                    let a_2 = attraction_radius * attraction_radius;
                    let distance_inside_2 = f32::max(0.0, (a_2 - r_2) / a_2);
                    center_sum += delta * distance_inside_2.powi(2);
                    center_n += 1.0;
                },
                CameraGuide::Center => {
                    center_sum += transform.translation.xy();
                    center_n += 1.0;
                    lowest = Some(lowest.unwrap_or(transform.translation.xy()).min(transform.translation.xy()));
                    highest = Some(highest.unwrap_or(transform.translation.xy()).max(transform.translation.xy()));
                },
                CameraGuide::MustBeOnscreen => {
                    lowest = Some(lowest.unwrap_or(transform.translation.xy()).min(transform.translation.xy()));
                    highest = Some(highest.unwrap_or(transform.translation.xy()).max(transform.translation.xy()));
                },
            }
        }

        if center_n > 0.0 {
            cam.target_center = center_sum / center_n;
        }

        let size = match (lowest, highest) {
            (Some(vl), Some(vh)) => {
                Vec2::max((vl - cam.target_center).abs(), (vh - cam.target_center).abs())
            },
            (Some(v), None) | (None, Some(v)) => {
                v - cam.target_center
            },
            (None, None) => Vec2::new(1.0, 1.0),
        };

        let size = Vec2::new(size.x, size.y / aspect);
        let size = f32::max(size.x, size.y);
        cam.target_size = Vec2::new(size, size);

        if cam.snap > 1.0 && (cam_trans.scale.xy() != cam.target_center || cam_trans.scale.xy() != cam.target_size) {
            cam.snap = 0.0;
        }
        
        let x = cam.snap.cubic_in();
        *cam_trans = TransLerp(*cam_trans).lerp(&TransLerp(Transform::from_translation(
            cam.target_center.extend(cam_trans.translation.z)).with_scale(cam.target_size.extend(cam_trans.scale.z))), &x).0;
        cam.snap += 1.0*time.delta_seconds();
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

    commands.entity(camera)
        .add_child(left)
        .add_child(right)
        .add_child(top)
        .add_child(bottom);
}

fn resize_borders(
    cameras: Query<(Entity, &LetterboxProjection, &Children), Changed<LetterboxProjection>>,
    mut borders: Query<(&mut Sprite, &mut Transform, &Border), Without<LetterboxProjection>>,
) {
    if let Ok((e, projection, children)) = cameras.get_single() {
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