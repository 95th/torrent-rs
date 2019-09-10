use dht::settings::DhtSettings;

fn test_settings() -> DhtSettings {
    let mut settings = DhtSettings::default();
    settings.max_torrents = 2;
    settings.max_dht_items = 2;
    settings.item_lifetime = 120 * 60;
    settings
}
