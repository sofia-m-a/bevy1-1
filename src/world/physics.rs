use bevy_rapier2d::prelude::Collider;

use super::feature::*;

pub fn collider_for(f: Feature) -> Option<Collider> {
    match f {
        Feature::GroundBlock(_, _, _)
        | Feature::Tile(_, _)
        | Feature::BigMushroomTop(_, _)
        | Feature::CrateCrossRect(_)
        | Feature::CrateRandomRect(_) => Some(Collider::cuboid(
            f.bounds().x.size() as f32 / 2.0,
            f.bounds().y.size() as f32 / 2.0,
        )),

        Feature::HillBlock { .. } => None,
        // Feature::HillBlock { terrain, start_x, height, bridge_thickness: None, lr } => Some({
        //     Collider::
        // }),
        // Feature::HillBlock { terrain, start_x, height, bridge_thickness: Some(t), lr } => Some({

        // }),
        Feature::Igloo { .. }
        | Feature::SurfaceWater(_)
        | Feature::SurfaceLava(_)
        | Feature::BigMushroomStem(_, _)
        | Feature::SlopedGround { .. }
        | Feature::FlatGround(_, _)
        | Feature::Zone(_, _)
        | Feature::Offscreen(_) => None,
    }
}
