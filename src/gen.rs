//use noise::pre;
use rand::prelude::*;
use rand_pcg::Pcg64;

use crate::grid::SparseGrid;

fn gen_platform() {
    let mut tr = thread_rng();
    let rng = Pcg64::from_rng(&mut tr);
}
