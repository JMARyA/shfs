pub mod file;
use shfs_api::filesystem_entry;
use std::collections::HashMap;

/// Cache for VolumeClient
pub struct Cache {
    pub entry_cache: HashMap<String, filesystem_entry::FilesystemEntry>,
}

impl Cache {
    pub fn new() -> Cache {
        return Cache {
            entry_cache: HashMap::new(),
        };
    }

    pub fn add_entry(&mut self, e: &filesystem_entry::FilesystemEntry) {
        self.entry_cache.insert(e.path.to_string(), e.clone());
    }

    pub fn get_entry(&self, path: &str) -> Option<&filesystem_entry::FilesystemEntry> {
        if self.entry_cache.contains_key(path) {
            return Some(self.entry_cache.get(path).unwrap());
        }
        return None;
    }
}
