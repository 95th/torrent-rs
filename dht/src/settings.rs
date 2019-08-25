use bencode::Value;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct DhtSettings {
    max_peers_reply: i64,
    search_branching: i64,
    max_fail_count: i64,
    max_torrents: i64,
    max_dht_items: i64,
    max_peers: i64,
    max_torrent_search_reply: i64,
    restrict_routing_ips: bool,
    restrict_search_ips: bool,
    extended_routing_table: bool,
    aggressive_lookups: bool,
    privacy_lookups: bool,
    enforce_node_id: bool,
    ignore_dark_internet: bool,
    block_timeout: i64,
    block_ratelimit: i64,
    read_only: bool,
    item_lifetime: i64,
    upload_rate_limit: i64,
    sample_infohashes_interval: i64,
    max_infohashes_sample_count: i64,
}

pub(crate) struct Settings {
    dht_settings: DhtSettings,
    prefer_verified_node_ids: bool,
}

macro_rules! read_int {
    ($settings: expr, $dict: expr, $key: ident) => {
        if let Some(value) = $dict.get(stringify!($key)) {
            $settings.$key = value.as_int().unwrap();
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
        $dict.insert(stringify!($key).to_owned(), Value::with_int($settings.$key));
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

        Value::with_map(dict)
    }

    pub fn read(node: &Value) -> DhtSettings {
        let mut settings = DhtSettings::default();

        let dict = match node {
            Value::Dict(d) => d,
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
