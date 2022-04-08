use bevy_rapier2d::prelude::{Point, SharedShape};

use super::brushes2::*;

pub fn mesh_for(t: Tile) -> Option<SharedShape> {
    match t {
        Tile::Terrain(_, tt) => match tt {
            TerrainTile::RoundLedge(_) => todo!(),
            TerrainTile::OverLedge(_) => todo!(),
            TerrainTile::SlopeLedge(_) => todo!(),
            TerrainTile::Slope(_) => todo!(),
            TerrainTile::SlopeInt(_) => todo!(),
            TerrainTile::RockSlope(_, _) => todo!(),

            TerrainTile::Cap(_) => None,

            TerrainTile::BlockLedge(_)
            | TerrainTile::BlockFace(_, _)
            | TerrainTile::FaceInt(_, _)
            | TerrainTile::Block
            | TerrainTile::BareBlock
            | TerrainTile::Single
            | TerrainTile::SingleBare => Some(SharedShape::trimesh(
                vec![
                    Point::new(-0.5, -0.5),
                    Point::new(-0.5, 0.5),
                    Point::new(0.5, 0.5),
                    Point::new(0.5, -0.5),
                ],
                vec![[0, 2, 1], [0, 3, 2]],
            )),

            TerrainTile::SingleHalf(_) | TerrainTile::Half(_, _) => todo!(),

            TerrainTile::Jagged => todo!(),
        },
        _ => None,
    }
}
