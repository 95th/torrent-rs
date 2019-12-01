use bencode::{Value, ValueRef};
use std::collections::BTreeMap;

/// structure used to hold configuration options for the DHT
pub struct DhtSettings {
    pub max_peers_reply: usize,
    pub search_branching: usize,
    pub max_fail_count: usize,
    pub max_torrents: usize,
    pub max_dht_items: usize,
    pub max_peers: usize,
    pub max_torrent_search_reply: usize,
    pub restrict_routing_ips: bool,
    pub restrict_search_ips: bool,
    pub extended_routing_table: bool,
    pub aggressive_lookups: bool,
    pub privacy_lookups: bool,
    pub enforce_node_id: bool,
    pub ignore_dark_internet: bool,
    pub block_timeout: usize,
    pub block_ratelimit: usize,
    pub read_only: bool,
    pub item_lifetime: usize,
    pub upload_rate_limit: usize,
    pub sample_infohashes_interval: usize,
    pub max_infohashes_sample_count: usize,
}

pub struct Settings {
    pub dht_settings: DhtSettings,
    pub prefer_verified_node_ids: bool,
}

macro_rules! read_int {
    ($settings: expr, $dict: expr, $key: ident) => {
        if let Some(value) = $dict.get(stringify!($key)) {
            $settings.$key = value.as_int().unwrap() as usize;
        }
    };
}

macro_rules! read_bool {
    ($settings: expr, $dict: expr, $key: ident) => {
        if let Some(value) = $dict.get(stringify!($key)) {
            $settings.$key = value.as_int().unwrap() != 0;
        }
    };
}

macro_rules! set_int {
    ($dict: expr, $key: ident, $settings: expr) => {
        $dict.insert(
            stringify!($key).to_owned(),
            Value::with_int($settings.$key as i64),
        );
    };
}

macro_rules! set_bool {
    ($dict: expr, $key: ident, $settings: expr) => {
        $dict.insert(
            stringify!($key).to_owned(),
            Value::with_int(if $settings.$key { 1 } else { 0 }),
        );
    };
}

impl Default for DhtSettings {
    fn default() -> Self {
        Self {
            max_peers_reply: 100,
            search_branching: 5,
            max_fail_count: 20,
            max_torrents: 2000,
            max_dht_items: 700,
            max_peers: 500,
            max_torrent_search_reply: 20,
            restrict_routing_ips: true,
            restrict_search_ips: true,
            extended_routing_table: true,
            aggressive_lookups: true,
            privacy_lookups: false,
            enforce_node_id: false,
            ignore_dark_internet: true,
            block_timeout: 5 * 60,
            block_ratelimit: 5,
            read_only: false,
            item_lifetime: 0,
            upload_rate_limit: 8000,
            sample_infohashes_interval: 21600,
            max_infohashes_sample_count: 20,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dht_settings: Default::default(),
            prefer_verified_node_ids: true,
        }
    }
}

impl DhtSettings {
    pub fn save(&self) -> Value {
        let mut dict = BTreeMap::new();

        set_int!(dict, max_peers_reply, self);
        set_int!(dict, search_branching, self);
        set_int!(dict, max_fail_count, self);
        set_int!(dict, max_torrents, self);
        set_int!(dict, max_dht_items, self);
        set_int!(dict, max_peers, self);
        set_int!(dict, max_torrent_search_reply, self);
        set_int!(dict, block_timeout, self);
        set_int!(dict, block_ratelimit, self);
        set_int!(dict, item_lifetime, self);

        set_bool!(dict, restrict_routing_ips, self);
        set_bool!(dict, restrict_search_ips, self);
        set_bool!(dict, extended_routing_table, self);
        set_bool!(dict, aggressive_lookups, self);
        set_bool!(dict, privacy_lookups, self);
        set_bool!(dict, enforce_node_id, self);
        set_bool!(dict, ignore_dark_internet, self);
        set_bool!(dict, read_only, self);

        Value::with_dict(dict)
    }

    pub fn read(node: &ValueRef) -> DhtSettings {
        let mut settings = DhtSettings::default();

        let dict = match node {
            ValueRef::Dict(d) => d,
            _ => return settings,
        };

        read_int!(settings, dict, max_peers_reply);
        read_int!(settings, dict, search_branching);
        read_int!(settings, dict, max_fail_count);
        read_int!(settings, dict, max_torrents);
        read_int!(settings, dict, max_dht_items);
        read_int!(settings, dict, max_peers);
        read_int!(settings, dict, max_torrent_search_reply);
        read_int!(settings, dict, block_timeout);
        read_int!(settings, dict, block_ratelimit);
        read_int!(settings, dict, item_lifetime);

        read_bool!(settings, dict, restrict_routing_ips);
        read_bool!(settings, dict, restrict_search_ips);
        read_bool!(settings, dict, extended_routing_table);
        read_bool!(settings, dict, aggressive_lookups);
        read_bool!(settings, dict, privacy_lookups);
        read_bool!(settings, dict, enforce_node_id);
        read_bool!(settings, dict, ignore_dark_internet);
        read_bool!(settings, dict, read_only);

        settings
    }
}
