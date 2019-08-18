use dht::id::Id;
use dht::id::ParseError;

#[test]
fn does_parse() {
    let hash = "00000000000000000000";
    let id: Id = hash.parse().unwrap();
    assert_eq!(Id::from([48; 20]), id);
}

#[test]
fn does_not_parse() {
    let hash = "0000";
    let id: Result<Id, ParseError> = hash.parse();
    assert!(id.is_err());
}

#[test]
fn xor() {
    let a: Id = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1].into();
    let b: Id = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2].into();
    let expected: Id = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3].into();
    assert_eq!(expected, &a ^ &b);
}

#[test]
fn at_dist() {
    let id: Id = [0; 20].into();
    let far = id.at_dist(6);
    let mut buf = [0; 20];
    buf[19] = 0x3F;
    let expected: Id = buf.into();
    assert_eq!(&expected, far.as_ref());
}

#[test]
fn get_dist() {
    let id: Id = [0; 20].into();
    let far = id.at_dist(6);
    assert_eq!(6, far.dist_to(&id));
}

#[test]
fn rand() {
    let lo = [0; 20].into();
    let hi = [1; 20].into();

    let random = Id::ranged_random(&lo, &hi);
    assert!(random >= lo);
    assert!(random < hi);
}
