use crate::detail;
use crate::node::NodeId;
use crate::settings::DhtSettings;

use bencode::Value;
use common::bloom_filter::{BloomFilter128, BloomFilter256};
use common::random;
use common::sha1::Sha1Hash;
use common::types::{PublicKey, SequenceNumber, Signature};

use std::cmp::Ordering;
use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

const TIME_DURATION: Duration = Duration::from_secs(30 * 60); // 30 minutes
const SAMPLE_INFOHASHES_INTERVAL_MAX: usize = 21600;
const INFOHASHES_SAMPLE_COUNT_MAX: usize = 20;
#[derive(Debug, Default, Copy, Clone)]
pub struct DhtStorageCounter {
    torrents: i32,
    peers: i32,
    immutable_data: i32,
    mutable_data: i32,
}

impl DhtStorageCounter {
    pub fn reset(&mut self) {
        self.torrents = 0;
        self.peers = 0;
        self.immutable_data = 0;
        self.mutable_data = 0;
    }
}

pub trait DhtStorage {
    fn update_node_ids(&mut self, ids: &[NodeId]);

    fn get_peers(
        &self,
        info_hash: &Sha1Hash,
        no_seed: bool,
        scrape: bool,
        requester: &SocketAddr,
        peers: &mut Value,
    ) -> io::Result<bool>;

    fn announce_peer(
        &mut self,
        info_hash: &Sha1Hash,
        endpoint: &SocketAddr,
        name: &str,
        seed: bool,
    );

    fn get_immutable_item(&self, target: &Sha1Hash, item: &mut Value) -> bool;

    fn put_immutable_item(&mut self, target: &Sha1Hash, item: &[u8], addr: &IpAddr);

    fn get_mutable_item_seq(&self, target: &Sha1Hash, seq: &mut SequenceNumber) -> bool;

    fn get_mutable_item(
        &self,
        target: &Sha1Hash,
        seq: SequenceNumber,
        force_fill: bool,
        item: &mut Value,
    ) -> bool;

    fn put_mutable_item(
        &mut self,
        target: &Sha1Hash,
        buf: &[u8],
        signature: &Signature,
        seq: SequenceNumber,
        pk: &PublicKey,
        salt: &[u8],
        address: &IpAddr,
    );

    fn get_infohashes_sample(&self, item: &mut Value) -> usize;

    fn tick(&mut self);

    fn counters(&self) -> DhtStorageCounter;
}

#[derive(PartialEq, Eq)]
struct PeerEntry {
    added: Instant,
    addr: SocketAddr,
    seed: bool,
}

impl PartialOrd for PeerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PeerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.addr
            .ip()
            .cmp(&other.addr.ip())
            .then(self.addr.port().cmp(&other.addr.port()))
    }
}

#[derive(Default)]
struct TorrentEntry {
    name: String,
    peers4: Vec<PeerEntry>,
    peers6: Vec<PeerEntry>,
}

trait ImmutableItem {
    fn num_announcers(&self) -> usize;
}

struct DhtImmutableItem {
    value: Vec<u8>,
    ips: BloomFilter128,
    last_seen: Instant,
    num_announcers: usize,
    size: usize,
}

impl Default for DhtImmutableItem {
    fn default() -> Self {
        Self {
            value: vec![],
            ips: BloomFilter128::default(),
            last_seen: Instant::now(),
            num_announcers: 0,
            size: 0,
        }
    }
}

#[derive(Default)]
struct DhtMutableItem {
    inner: DhtImmutableItem,
    sig: Signature,
    seq: SequenceNumber,
    key: PublicKey,
    salt: Vec<u8>,
}

impl DhtMutableItem {
    pub fn new(value: Vec<u8>) -> Self {
        Self {
            inner: DhtImmutableItem::new(value),
            ..Default::default()
        }
    }
}

impl DhtImmutableItem {
    pub fn new(value: Vec<u8>) -> DhtImmutableItem {
        DhtImmutableItem {
            value,
            ips: BloomFilter128::new(),
            last_seen: Instant::now(),
            num_announcers: 0,
            size: 0,
        }
    }

    fn set_value(&mut self, buf: &[u8]) {
        if self.value.len() != buf.len() {
            self.value = buf.to_vec();
        } else {
            self.value.copy_from_slice(buf);
        }
    }

    pub fn touch_item(&mut self, addr: &IpAddr) {
        self.last_seen = Instant::now();
        let iphash = Sha1Hash::from_address(addr);
        if !self.ips.find(&iphash) {
            self.ips.set(&iphash);
            self.num_announcers += 1;
        }
    }
}

impl ImmutableItem for DhtImmutableItem {
    fn num_announcers(&self) -> usize {
        self.num_announcers
    }
}

impl ImmutableItem for DhtMutableItem {
    fn num_announcers(&self) -> usize {
        self.inner.num_announcers
    }
}

pub struct InfohashesSample {
    samples: Vec<Sha1Hash>,
    created: Instant,
}

impl InfohashesSample {
    pub fn new() -> InfohashesSample {
        InfohashesSample {
            samples: vec![],
            // TODO: Is this correct? See dht_storage.cpp Line:183
            created: Instant::now(),
        }
    }

    pub fn count(&self) -> usize {
        self.samples.len()
    }
}

pub struct DefaultDhtStorage<'a> {
    settings: &'a DhtSettings,
    counters: DhtStorageCounter,
    node_ids: Vec<NodeId>,
    map: HashMap<NodeId, TorrentEntry>,
    immutable_table: HashMap<NodeId, DhtImmutableItem>,
    mutable_table: HashMap<NodeId, DhtMutableItem>,
    infohashes_sample: InfohashesSample,
}

impl DefaultDhtStorage<'_> {
    pub fn new(settings: &DhtSettings) -> DefaultDhtStorage {
        DefaultDhtStorage {
            settings,
            counters: DhtStorageCounter::default(),
            node_ids: vec![],
            map: HashMap::new(),
            immutable_table: HashMap::new(),
            mutable_table: HashMap::new(),
            infohashes_sample: InfohashesSample::new(),
        }
    }
}

impl DhtStorage for DefaultDhtStorage<'_> {
    fn update_node_ids(&mut self, ids: &[NodeId]) {
        self.node_ids = ids.iter().cloned().collect();
    }

    fn get_peers(
        &self,
        info_hash: &Sha1Hash,
        no_seed: bool,
        scrape: bool,
        requester: &SocketAddr,
        peers: &mut Value,
    ) -> io::Result<bool> {
        let v = match self.map.get(info_hash) {
            Some(v) => v,
            None => return Ok(self.map.len() >= self.settings.max_torrents),
        };

        let peersv = if requester.is_ipv4() {
            &v.peers4
        } else {
            &v.peers6
        };

        let peer_map = peers
            .as_dict_mut()
            .ok_or_else(|| bencode::Error::ParseDict)?;

        if !v.name.is_empty() {
            peer_map.insert(String::from("n"), Value::with_str(&v.name));
        }

        if scrape {
            let mut downloaders = BloomFilter256::new();
            let mut seeds = BloomFilter256::new();

            for p in peersv {
                let ip_hash = Sha1Hash::from_address(&p.addr.ip());
                if p.seed {
                    seeds.set(&ip_hash);
                } else {
                    downloaders.set(&ip_hash);
                }
            }

            peer_map.insert(String::from("BFpe"), downloaders.as_bytes().into());
            peer_map.insert(String::from("BFsd"), seeds.as_bytes().into());
        } else {
            let mut to_pick = self.settings.max_peers_reply;

            if !peersv.is_empty() && requester.is_ipv6() {
                to_pick /= 4;
            }

            let pe = peer_map
                .get_mut("values")
                .unwrap()
                .as_list_mut()
                .ok_or_else(|| bencode::Error::ParseList)?;

            let mut candidates = peersv.iter().filter(|v| !(no_seed && v.seed)).count();
            to_pick = to_pick.min(candidates);

            let mut iter = peersv.iter();
            while to_pick > 0 {
                let p = iter.next().unwrap();
                if no_seed && p.seed {
                    continue;
                }

                assert!(candidates >= to_pick);
                candidates -= 1;
                if random::random_usize(candidates + 1) > to_pick {
                    continue;
                }

                let mut buf = Vec::new();
                detail::write_socket_addr(&mut buf, &p.addr)?;
                pe.push(buf.into());

                to_pick -= 1;
            }
        }

        if peersv.len() < self.settings.max_peers {
            return Ok(false);
        }

        let requester_entry = PeerEntry {
            added: Instant::now(),
            addr: *requester,
            seed: false,
        };

        match peersv.binary_search(&requester_entry) {
            Ok(i) => Ok(&peersv[i].addr != requester),
            Err(_) => Ok(true),
        }
    }

    fn announce_peer(
        &mut self,
        info_hash: &Sha1Hash,
        endpoint: &SocketAddr,
        name: &str,
        seed: bool,
    ) {
        let v = if let Some(v) = self.map.get_mut(info_hash) {
            v
        } else {
            if self.map.len() > self.settings.max_torrents {
                // we're at capacity, drop the announce
                return;
            }

            self.counters.torrents += 1;
            self.map.entry(info_hash.clone()).or_default()
        };

        // the peer announces a torrent name, and we don't have a name
        // for this torrent. Store it.
        if !name.is_empty() && v.name.is_empty() {
            v.name = name.chars().take(100).collect();
        }

        let peersv = if endpoint.is_ipv4() {
            &mut v.peers4
        } else {
            &mut v.peers6
        };

        let peer = PeerEntry {
            addr: endpoint.clone(),
            added: Instant::now(),
            seed,
        };

        match peersv.binary_search(&peer) {
            Ok(i) if &peersv[i].addr == endpoint => peersv[i] = peer,
            v => {
                if peersv.len() >= self.settings.max_peers {
                    // we're at capacity, drop the announce
                    return;
                } else {
                    let i = v.unwrap_or_else(|x| x);
                    peersv.insert(i, peer);
                    self.counters.peers += 1;
                }
            }
        }
    }

    fn get_immutable_item(&self, target: &Sha1Hash, item: &mut Value) -> bool {
        if let Some(dht) = self.immutable_table.get(target) {
            if let Some(dict) = item.as_dict_mut() {
                if let Ok(v) = Value::decode(&dht.value) {
                    dict.insert("v".to_string(), v);
                } else {
                    debug_assert!(false, "Unable to decode");
                }
            } else {
                debug_assert!(false, "item is not a dict");
            }
            true
        } else {
            false
        }
    }

    fn put_immutable_item(&mut self, target: &Sha1Hash, item: &[u8], addr: &IpAddr) {
        debug_assert!(!self.node_ids.is_empty());
        let mut i = self.immutable_table.get_mut(target);
        if i.is_none() {
            if self.immutable_table.len() >= self.settings.max_dht_items {
                let (k, _) = pick_least_imp(&self.node_ids, &self.immutable_table).unwrap();
                let k = k.clone();
                self.immutable_table.remove(&k);
                self.counters.immutable_data -= 1;
            }

            let to_add = DhtImmutableItem::new(item.to_owned());
            self.immutable_table.insert(target.clone(), to_add);
            i = self.immutable_table.get_mut(target);
            self.counters.immutable_data += 1;
        }

        i.unwrap().touch_item(addr);
    }

    fn get_mutable_item_seq(&self, target: &Sha1Hash, seq: &mut SequenceNumber) -> bool {
        if let Some(item) = self.mutable_table.get(target) {
            *seq = item.seq;
            true
        } else {
            false
        }
    }

    fn get_mutable_item(
        &self,
        target: &Sha1Hash,
        seq: SequenceNumber,
        force_fill: bool,
        item: &mut Value,
    ) -> bool {
        if let Some(f) = self.mutable_table.get(target) {
            if let Some(dict) = item.as_dict_mut() {
                dict.insert("seq".to_string(), Value::with_int(f.seq));
                if force_fill || 0 <= seq && seq < f.seq {
                    dict.insert("v".to_string(), Value::decode(&f.inner.value).unwrap());
                    dict.insert("sig".to_string(), Value::from(&f.sig[..]));
                    dict.insert("k".to_string(), Value::from(&f.key[..]));
                }
            } else {
                panic!("Value not a dict");
            }
            true
        } else {
            false
        }
    }

    fn put_mutable_item(
        &mut self,
        target: &Sha1Hash,
        buf: &[u8],
        signature: &Signature,
        seq: SequenceNumber,
        pk: &PublicKey,
        salt: &[u8],
        addr: &IpAddr,
    ) {
        debug_assert!(!self.node_ids.is_empty());
        match self.mutable_table.get_mut(target) {
            Some(item) => {
                if item.seq < seq {
                    item.inner.set_value(buf);
                    item.seq = seq;
                    item.sig = signature.clone();
                }
                item.inner.touch_item(addr);
            }
            None => {
                // this is the case where we don't have an item in this slot
                // make sure we don't add too many items
                if self.mutable_table.len() >= self.settings.max_dht_items {
                    let (key, _) = pick_least_imp(&self.node_ids, &self.mutable_table).unwrap();
                    let key = key.clone();
                    self.mutable_table.remove(&key);
                    self.counters.mutable_data += 1;
                }
                let mut item = DhtMutableItem::new(buf.to_vec());
                item.seq = seq;
                item.salt = salt.to_vec();
                item.sig = signature.clone();
                item.key = pk.clone();
                item.inner.touch_item(addr);

                self.mutable_table.insert(target.clone(), item);
                self.counters.mutable_data += 1;
            }
        }
    }

    fn get_infohashes_sample(&self, item: &mut Value) -> usize {
        let dict = item.as_dict_mut().unwrap();
        let interval = common::clamp(
            self.settings.sample_infohashes_interval,
            0,
            SAMPLE_INFOHASHES_INTERVAL_MAX,
        );
        dict.insert("interval".to_string(), Value::with_int(interval as i64));
        dict.insert("num".to_string(), Value::with_int(self.map.len() as i64));
        unimplemented!()
    }

    fn tick(&mut self) {
        unimplemented!()
    }

    fn counters(&self) -> DhtStorageCounter {
        unimplemented!()
    }
}

fn pick_least_imp<'a, T: ImmutableItem>(
    node_ids: &[NodeId],
    table: &'a HashMap<NodeId, T>,
) -> Option<(&'a NodeId, &'a T)> {
    table.iter().min_by(|(node_1, t1), (node_2, t2)| {
        let dist_1 = crate::node::min_distance_exp(node_1, node_ids);
        let dist_2 = crate::node::min_distance_exp(node_2, node_ids);

        let n1 = t1.num_announcers() / 5 - dist_1;
        let n2 = t2.num_announcers() / 5 - dist_2;
        n1.cmp(&n2)
    })
}
