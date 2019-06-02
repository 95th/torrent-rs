use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

pub struct Storage {
    map: HashMap<String, (Instant, String)>,
    ttl: Duration,
}

impl Storage {
    pub fn get(&mut self, key: &str) -> Option<&String> {
        self.cull();
        self.map.get(key).map(|(_, v)| v)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.cull();
        self.map
            .insert(key.to_owned(), (Instant::now(), value.to_owned()));
    }

    pub fn all(&mut self) -> impl Iterator<Item = (&String, &String)> {
        self.cull();
        self.map.iter().map(|(k, (_, v))| (k, v))
    }

    fn cull(&mut self) {
        let instant = Instant::now() - self.ttl;
        self.map.retain(|_, (i, _)| *i > instant);
    }

    pub fn iter(&mut self) -> impl Iterator<Item = (&String, &String)> {
        self.cull();
        self.map.iter().map(|(k, (_i, v))| (k, v))
    }
}
