use bencode::BorrowValue;
use dht::node::NodeId;
use dht::state::DhtState;

fn main() {
    let mut state = DhtState::default();
    state.nids.push((
        "100.100.100.100".parse().unwrap(),
        NodeId::from_bytes(b"aaaaabbbbbcccccddddd"),
    ));

    let value = state.save();
    println!("{}", value);
    let bytes = value.to_vec();
    let borrowed = BorrowValue::decode(&bytes).unwrap();

    let state = DhtState::read(&borrowed).unwrap();
    println!("{:?}", state);
}
