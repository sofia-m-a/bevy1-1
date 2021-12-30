use bevy::prelude::*;
use bevy_tilemap::{tilemap::TilemapResult, Tilemap};
use rand::prelude::*;
use rand_pcg::Pcg64;

use crate::{assets::SpriteAssets, brushes::Tile};

pub fn generate_island(map: &mut Tilemap) -> TilemapResult<()> {
    let mut tr = thread_rng();
    let rng = Pcg64::from_rng(&mut tr);

    let mut tiles = Vec::new();
    for i in 0..=10 {
        for j in 0..=10 {
            tiles.push(bevy_tilemap::tile::Tile {
                point: (i, j),
                sprite_order: 0,
                sprite_index: 27 * 21,
                ..Default::default()
            });
        }
    }

    //map.insert_chunk((0, 0))?;
    map.insert_tiles(tiles)?;
    println!("{:?}", map);
    Ok(())
}
