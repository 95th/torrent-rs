use dht::state::DhtState;

fn main() {
    let mut state = DhtState::default();
    state.nids.push((
        "100.100.100.100".parse().unwrap(),
        b"aaaaabbbbbcccccddddd".to_vec(),
    ));

    let value = state.save();
    println!("{}", value);

    let state = DhtState::read(&value).unwrap();
    println!("{:?}", state);
}
