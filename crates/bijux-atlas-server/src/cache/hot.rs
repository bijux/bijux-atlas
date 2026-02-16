use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct HotEntry {
    pub body: Vec<u8>,
    pub etag: String,
    pub created_at: Instant,
}

pub struct HotQueryCache {
    ttl: Duration,
    max_entries: usize,
    entries: HashMap<String, HotEntry>,
}

impl HotQueryCache {
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            ttl,
            max_entries,
            entries: HashMap::new(),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<HotEntry> {
        self.entries
            .retain(|_, v| v.created_at.elapsed() <= self.ttl);
        self.entries.get(key).cloned()
    }

    pub fn insert(&mut self, key: String, value: HotEntry) {
        self.entries
            .retain(|_, v| v.created_at.elapsed() <= self.ttl);
        if self.entries.len() >= self.max_entries {
            if let Some(victim) = self
                .entries
                .iter()
                .min_by_key(|(_, v)| v.created_at)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&victim);
            }
        }
        self.entries.insert(key, value);
    }
}
