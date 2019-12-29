use bencode::ValueRef;
use common::sha1::Sha1Hash;

#[test]
fn parse_torrent_file() {
    let f: &[u8] = include_bytes!("update.torrent");
    let val = ValueRef::decode(f).unwrap();

    let dict = val.as_dict().unwrap();
    assert_eq!(
        "http://0.0.0.0:52585/announce",
        dict["announce"].as_str().unwrap()
    );
    assert_eq!("update.txt", dict["comment"].as_str().unwrap());
    assert_eq!("BitTorrent/7.10.5", dict["created by"].as_str().unwrap());
    assert_eq!(1577614375, dict["creation date"].as_int().unwrap());
    assert_eq!("UTF-8", dict["encoding"].as_str().unwrap());

    let info = dict["info"].as_dict().unwrap();

    assert_eq!(359, info["length"].as_int().unwrap());
    assert_eq!("update.txt", info["name"].as_str().unwrap());
    assert_eq!(16384, info["piece length"].as_int().unwrap());
    let pieces = info["pieces"].as_bytes().unwrap();
    assert_eq!(20, pieces.len()); // Only one piece
    let hash = Sha1Hash::from_bytes(pieces).unwrap();
    assert_eq!("07691fe65b4c51db229460ba527eb1c7487f2477", hash.to_string());
}
