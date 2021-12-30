//use noise::pre;
use rand::prelude::*;
use rand_pcg::Pcg64;

fn gen_platform() {
    let mut tr = thread_rng();
    let rng = Pcg64::from_rng(&mut tr);
}
