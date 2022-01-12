use petgraph::{dot::Dot, graph::NodeIndex, Graph};
use rand::{thread_rng, Rng, SeedableRng};
use rand_pcg::Pcg64;

#[derive(Clone, Copy, Debug)]
enum Zone {
    Flat,
}

fn gen_graph(rng: &mut Pcg64) -> Graph<Zone, i32> {
    let mut g = Graph::new();
    let start = g.add_node(Zone::Flat);
    let mut main_path = start;
    for i in 0..(100 as i32) {
        let new = g.add_node(Zone::Flat);
        g.add_edge(main_path, new, 3);

        if rng.gen_bool(0.1) {
            g.add_edge(
                NodeIndex::new(rng.gen_range(0.max(i - 10)..i) as usize),
                new,
                2,
            );
        }

        main_path = new;
    }

    for _ in 0..rng.gen_range(1..20) {
        let start = g.add_node(Zone::Flat);
        let mut end = start;
        for _ in 0..rng.gen_range(1..6) {
            let new = g.add_node(Zone::Flat);
            g.add_edge(end, new, 4);
            end = new;
        }

        let start_index = rng.gen_range(0..100);
        g.add_edge(NodeIndex::new(start_index), start, 5);
        let end_index = rng.gen_range(start_index..100.min(start_index + 15));
        g.add_edge(end, NodeIndex::new(end_index), 5);
    }

    g
}

pub fn debug_graph() {
    let mut tr = thread_rng();
    let mut rng = Pcg64::from_rng(&mut tr).unwrap();
    let graph = gen_graph(&mut rng);
    println!("{:?}", Dot::new(&graph.map(|_, n| { 0u32 }, |_, e| { e })));
}
