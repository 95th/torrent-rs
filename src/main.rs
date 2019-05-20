use std::time::Instant;

use dht::NodeId;

fn main() {
    let id: NodeId = "AAAAAAAAAAAAAAAAAAAA".parse().unwrap();
    println!("{}", id);
    for _ in 1..100 {
        let start = Instant::now();
        for _ in 1..10000 {
            let _list: Vec<NodeId> = (1..160).map(|i| id.at_dist(i)).collect();
        }
        let took = Instant::now() - start;
        println!("{} ms", took.as_millis());
    }
}