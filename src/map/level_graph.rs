use bevy::math::Vec2;
use petgraph::{dot::Dot, graph::NodeIndex, Graph};
use rand::{
    distributions::{self, WeightedIndex},
    prelude::Distribution,
    thread_rng, Rng, SeedableRng,
};
use rand_pcg::Pcg64;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum Zone {
    Flat,
    Slopes,
    Pits,
    Caves,
}

#[derive(Clone, Copy, Debug)]
pub struct ZoneInfo {
    kind: Zone,
    bonus: bool,
    powerup_block: bool,
    decorate: bool,
    safe: bool,
}

fn start_zone() -> ZoneInfo {
    ZoneInfo {
        kind: Zone::Flat,
        bonus: false,
        powerup_block: true,
        decorate: true,
        safe: true,
    }
}

fn gen_zone(rng: &mut Pcg64) -> ZoneInfo {
    let kind: WeightedIndex<usize> = WeightedIndex::new([4, 3, 2, 1]).unwrap();
    let kind = [Zone::Flat, Zone::Slopes, Zone::Pits, Zone::Caves][kind.sample(rng)];
    let bonus = rng.gen_bool(0.1);
    let powerup_block = rng.gen_bool(0.2);
    let decorate = rng.gen_bool(0.8);
    let safe = rng.gen_bool(0.3);

    ZoneInfo {
        kind,
        bonus,
        powerup_block,
        decorate,
        safe,
    }
}

pub fn gen_graph(rng: &mut Pcg64) -> Graph<ZoneInfo, i32> {
    let mut g = Graph::new();
    let start = g.add_node(start_zone());
    let mut main_path = start;
    for i in 0..(100 as i32) {
        let new = g.add_node(gen_zone(rng));
        g.add_edge(main_path, new, 3);

        if rng.gen_bool(0.3) {
            g.add_edge(
                NodeIndex::new(rng.gen_range(0.max(i - 10)..1.max(i)) as usize),
                new,
                2,
            );
        }

        main_path = new;
    }

    for _ in 0..rng.gen_range(1..20) {
        let start = g.add_node(gen_zone(rng));
        let mut end = start;
        for _ in 0..rng.gen_range(1..6) {
            let new = g.add_node(gen_zone(rng));
            g.add_edge(end, new, 4);
            end = new;
        }

        let start_index = rng.gen_range(0..100);
        g.add_edge(NodeIndex::new(start_index), start, 5);
        let end_index = rng.gen_range(start_index..100.min(start_index + 15));
        g.add_edge(end, NodeIndex::new(end_index), 5);
    }

    for _ in 0..rng.gen_range(1..10) {
        let start = g.add_node(start_zone());
        let mut end = start;
        for _ in 0..rng.gen_range(1..6) {
            let new = g.add_node(start_zone());
            g.add_edge(end, new, 4);
            end = new;
        }

        let start_index = rng.gen_range(0..100);
        g.add_edge(start, NodeIndex::new(start_index), 5);
        let end_index = rng.gen_range(start_index..100.min(start_index + 15));
        g.add_edge(NodeIndex::new(end_index), end, 5);
    }

    g
}

pub fn debug_graph() {
    let mut tr = thread_rng();
    let mut rng = Pcg64::from_rng(&mut tr).unwrap();
    let graph = gen_graph(&mut rng);
    println!("{:?}", Dot::new(&layout_graph(graph)));
}

fn inv(x: f32) -> f32 {
    if x == 0.0 {
        0.0
    } else {
        1.0 / x
    }
}

fn layout_iteration(positions: &mut Vec<Vec2>, distances: &HashMap<(usize, usize), f32>) {
    for i in 0..positions.len() {
        let denominator = positions.len() - 1;
        let mut sum = Vec2::ZERO;
        for (j, &p) in positions.iter().enumerate() {
            if i == j {
                continue;
            }
            sum += positions[j]
                + (positions[i] - positions[j])
                    * *distances.get(&(i, j)).unwrap_or(&10.0)
                    * inv((positions[i] - positions[j]).length());
        }
        positions[i] = sum / denominator as f32;
    }
}

pub fn layout_graph(graph: Graph<ZoneInfo, i32>) -> Graph<(ZoneInfo, Vec2), f32> {
    let map = graph
        .raw_edges()
        .iter()
        .map(|e| ((e.source().index(), e.target().index()), e.weight as f32))
        .collect::<HashMap<(usize, usize), f32>>();

    let mut pcg = thread_rng();
    let mut v = Vec::new();
    for i in 0..graph.node_count() {
        let x: f32 = pcg.sample(distributions::OpenClosed01);
        let y: f32 = pcg.sample(distributions::OpenClosed01);
        v.push(Vec2::new(100.0 * x, 100.0 * y));
    }

    for i in 0..100 {
        layout_iteration(&mut v, &map);
    }

    graph.map(|i, n| (*n, v[i.index()]), |_, e| *e as f32)
}
