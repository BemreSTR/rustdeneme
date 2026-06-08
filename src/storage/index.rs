use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct EntryIndex {
    entries: BTreeMap<String, String>,
}

impl EntryIndex {
    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        self.entries.insert(key, value)
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.entries.remove(key)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.entries.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn as_map(&self) -> &BTreeMap<String, String> {
        &self.entries
    }
}
