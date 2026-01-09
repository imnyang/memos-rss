use anyhow::Result;
use sled::Db;
use std::collections::HashSet;

pub struct Storage {
    db: Db,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn is_processed(&self, feed_name: &str, item_id: &str) -> Result<bool> {
        let key = format!("{}:{}", feed_name, item_id);
        Ok(self.db.get(key)?.is_some())
    }

    pub fn mark_processed(&self, feed_name: &str, item_id: &str) -> Result<()> {
        let key = format!("{}:{}", feed_name, item_id);
        self.db.insert(key, b"")?;
        Ok(())
    }

    pub fn mark_processed_bulk(&self, data: std::collections::HashMap<String, Vec<String>>) -> Result<()> {
        let mut batch = sled::Batch::default();
        for (feed_name, ids) in data {
            for id in ids {
                let key = format!("{}:{}", feed_name, id);
                batch.insert(key.as_bytes(), b"");
            }
        }
        self.db.apply_batch(batch)?;
        Ok(())
    }

    pub fn get_processed_ids(&self, feed_name: &str) -> Result<HashSet<String>> {
        let prefix = format!("{}:", feed_name);
        let mut ids = HashSet::new();
        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, _) = item?;
            let key_str = String::from_utf8(key.to_vec())?;
            if let Some(id) = key_str.strip_prefix(&prefix) {
                ids.insert(id.to_string());
            }
        }
        Ok(ids)
    }

    pub fn remove_processed(&self, feed_name: &str, item_id: &str) -> Result<()> {
        let key = format!("{}:{}", feed_name, item_id);
        self.db.remove(key)?;
        Ok(())
    }
}
