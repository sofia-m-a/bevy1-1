use bevy::{math::IVec2, prelude::*};
use extent::Extent;

use crate::{
    assets::{SpriteAssets, TILE_SIZE},
    camera::SofiaCamera,
};

use super::brushes::{Terrain, TerrainTheme, Tile};

// must be a power of two for Morton encoding to work
// otherwise we need to change CHUNK_SIZE^2 below to to_nearest_pow2(CHUNK_SIZE)^2
// pub const CHUNK_SIZE: usize = 32;

pub type Place = IVec2;

pub struct Map {}

#[derive(Default, Component)]
pub struct MapView {
    chunks_l: Vec<(i32, Entity)>,
    chunks_r: Vec<(i32, Entity)>,
    chunks_b: Vec<(i32, Entity)>,
    chunks_t: Vec<(i32, Entity)>,
}

#[derive(Component)]
pub struct Chunk {
    width: Extent<i32>,
    height: Extent<i32>,
    tiles: Vec<(Place, Tile)>,
    layer: u8,
}

impl MapView {
    fn insert_chunk(
        &mut self,
        commands: &mut Commands,
        chunk: Chunk,
        trans: Transform,
        sa: &Res<SpriteAssets>,
    ) -> Entity {
        let id = commands.spawn().id();
        if let (Some(l), Some(r), Some(b), Some(t)) = (
            chunk.width.lo(),
            chunk.width.hi(),
            chunk.height.lo(),
            chunk.height.hi(),
        ) {
            self.chunks_l.push((l, id));
            self.chunks_r.push((r, id));
            self.chunks_b.push((t, id));
            self.chunks_t.push((b, id));
        }
        let mut entities = Vec::new();
        for (p, t) in chunk.tiles.iter().copied() {
            entities.push(
                commands
                    .spawn()
                    .insert_bundle(SpriteSheetBundle {
                        sprite: TextureAtlasSprite {
                            index: u16::from(t) as usize,
                            ..Default::default()
                        },
                        texture_atlas: sa.tile_texture.clone(),
                        transform: trans.mul_transform(Transform::from_translation(Vec3::new(
                            p.x as f32 * TILE_SIZE as f32,
                            p.y as f32 * TILE_SIZE as f32,
                            chunk.layer as f32,
                        ))),
                        ..Default::default()
                    })
                    .id(),
            );
        }
        commands
            .entity(id)
            .insert(trans)
            .push_children(&entities)
            .insert(chunk);

        id
    }

    fn remove_chunk(&mut self, commands: &mut Commands, chunk: Entity) {
        self.chunks_l.retain(|&(_, e)| e != chunk);
        self.chunks_r.retain(|&(_, e)| e != chunk);
        self.chunks_b.retain(|&(_, e)| e != chunk);
        self.chunks_t.retain(|&(_, e)| e != chunk);
        commands.entity(chunk).despawn_recursive();
    }
}

pub fn chunk_system(
    mut commands: Commands,
    mut maps: Query<&mut MapView>,
    views: Query<&SofiaCamera>,
    sa: Res<SpriteAssets>,
) {
    if let Some(view) = views.iter().next() {
        for mut map in maps.iter_mut() {
            let mut to_remove = Vec::new();
            for &(l, e) in map.chunks_l.iter() {
                if l as f32 > view.view.right {
                    to_remove.push(e);
                }
            }
            for &(r, e) in map.chunks_r.iter() {
                if (r as f32) < view.view.left {
                    to_remove.push(e);
                }
            }
            for &(b, e) in map.chunks_b.iter() {
                if b as f32 > view.view.top {
                    to_remove.push(e);
                }
            }
            for &(t, e) in map.chunks_t.iter() {
                if (t as f32) < view.view.bottom {
                    to_remove.push(e);
                }
            }

            for &e in to_remove.iter() {
                map.remove_chunk(&mut commands, e);
            }

            map.insert_chunk(
                &mut commands,
                load_chunks(
                    Extent::new(view.view.left.floor() as i32, view.view.right.ceil() as i32),
                    Extent::new(view.view.bottom.floor() as i32, view.view.top.ceil() as i32),
                ),
                Transform::from_translation(
                    Vec2::new(view.view.left, view.view.bottom).extend(0.0),
                )
                .with_scale(Vec2::splat(TILE_SIZE as f32).extend(1.0)),
                &sa,
            );
        }
    }
}

pub fn load_chunks(width: Extent<i32>, height: Extent<i32>) -> Chunk {
    let mut c = Chunk {
        width,
        height,
        tiles: Vec::new(),
        layer: 1,
    };
    dbg!(width, height);
    for i in width.iter() {
        for j in height.iter() {
            c.tiles.push((
                Place::new(i, j),
                Tile::Ground(Terrain::Block, TerrainTheme::Choco),
            ));
        }
    }
    c
}
