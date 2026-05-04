use std::collections::HashSet;
use tokio::sync::RwLock;

pub struct IdempotencyCache {
    seen: RwLock<HashSet<String>>,
    max_size: usize,
}

impl IdempotencyCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            seen: RwLock::new(HashSet::new()),
            max_size,
        }
    }

    pub async fn check_and_record(&self, key: &str) -> bool {
        let mut seen = self.seen.write().await;
        if seen.contains(key) {
            return true;
        }
        if seen.len() >= self.max_size {
            seen.clear();
        }
        seen.insert(key.to_string());
        false
    }
}
