use crate::bloom_filter::BloomFilter;
use crate::detail;
use crate::node::NodeId;
use crate::rand;
use crate::settings::DhtSettings;
use crate::types::{SequenceNumber, Signature};
use crate::Sha1Hash;

use bencode::Value;

use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

const TIME_DURATION: Duration = Duration::from_secs(30 * 60); // 30 minutes

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
    #[cfg(feature = "abi_v1")]
    fn num_torrents(&self) -> usize;

    #[cfg(feature = "abi_v1")]
    fn num_peers(&self) -> usize;

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

    fn get_immutable_item(&self, target: &Sha1Hash) -> Option<Value>;

    fn put_immutable_item(&mut self, target: &Sha1Hash, item: &Value, addr: &IpAddr);

    fn get_mutable_item_seq(&self, target: &Sha1Hash) -> SequenceNumber;

    fn get_mutable_item(
        &self,
        target: &Sha1Hash,
        seq: SequenceNumber,
        force_fill: bool,
    ) -> Option<Value>;

    fn put_mutable_item(
        &mut self,
        target: &Sha1Hash,
        item: &Value,
        signature: &Signature,
        seq: SequenceNumber,
        force_fill: bool,
    );

    fn get_infohashes_sample(&self, item: &Value) -> usize;

    fn tick(&mut self);

    fn counters(&self) -> DhtStorageCounter;
}

struct PeerEntry {
    added: Instant,
    addr: SocketAddr,
    seed: bool,
}

struct TorrentEntry {
    name: String,
    peers4: Vec<PeerEntry>,
    peers6: Vec<PeerEntry>,
}

struct DhtImmutableItem {
    value: String,
    ips: BloomFilter,
    last_seen: Instant,
    num_announcers: usize,
}

impl DhtImmutableItem {
    pub fn new(value: String) -> DhtImmutableItem {
        DhtImmutableItem {
            value,
            ips: BloomFilter::new(128),
            last_seen: Instant::now(),
            num_announcers: 0,
        }
    }

    pub fn set_value(&mut self, value: &str) {
        self.value = value.to_owned();
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
    mutable_table: HashMap<NodeId, DhtImmutableItem>,
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
    #[cfg(feature = "abi_v1")]
    fn num_torrents(&self) -> usize {
        self.map.len()
    }

    #[cfg(feature = "abi_v1")]
    fn num_peers(&self) -> usize {
        self.map
            .values()
            .map(|v| v.peers4.len() + v.peers6.len())
            .sum()
    }

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

        let peer_map = peers.as_map_mut()?;

        if !v.name.is_empty() {
            peer_map.insert(String::from("n"), Value::with_str(&v.name));
        }

        if scrape {
            let mut downloaders = BloomFilter::new(256);
            let mut seeds = BloomFilter::new(256);

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

            let pe = peer_map.get_mut("values").unwrap().as_list_mut()?;

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
                if rand::random(candidates + 1) > to_pick {
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

        let lower_bound_idx = detail::lower_bound(peersv, &requester_entry);

        Ok(false)
    }

    fn announce_peer(
        &mut self,
        info_hash: &Sha1Hash,
        endpoint: &SocketAddr,
        name: &str,
        seed: bool,
    ) {
        unimplemented!()
    }

    fn get_immutable_item(&self, target: &Sha1Hash) -> Option<Value> {
        unimplemented!()
    }

    fn put_immutable_item(&mut self, target: &Sha1Hash, item: &Value, addr: &IpAddr) {
        unimplemented!()
    }

    fn get_mutable_item_seq(&self, target: &Sha1Hash) -> SequenceNumber {
        unimplemented!()
    }

    fn get_mutable_item(
        &self,
        target: &Sha1Hash,
        seq: SequenceNumber,
        force_fill: bool,
    ) -> Option<Value> {
        unimplemented!()
    }

    fn put_mutable_item(
        &mut self,
        target: &Sha1Hash,
        item: &Value,
        signature: &Signature,
        seq: SequenceNumber,
        force_fill: bool,
    ) {
        unimplemented!()
    }

    fn get_infohashes_sample(&self, item: &Value) -> usize {
        unimplemented!()
    }

    fn tick(&mut self) {
        unimplemented!()
    }

    fn counters(&self) -> DhtStorageCounter {
        unimplemented!()
    }
}
