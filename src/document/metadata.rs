use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetadataValue {
    String(String),
    Number(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Metadata {
    inner: BTreeMap<String, MetadataValue>,
}

impl Metadata {
    pub fn new() -> Self {
        Metadata {
            inner: BTreeMap::new(),
        }
    }

    pub fn insert_string(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner.insert(key.into(), MetadataValue::String(value.into()));
    }

    pub fn insert_number(&mut self, key: impl Into<String>, value: i64) {
        self.inner.insert(key.into(), MetadataValue::Number(value));
    }
    
    // Helper to merge another metadata into this one (overriding common keys)
    pub fn merge(&mut self, other: Metadata) {
        for (k, v) in other.inner {
            self.inner.insert(k, v);
        }
    }

    pub fn get(&self, key: &str) -> Option<&MetadataValue> {
        self.inner.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &MetadataValue)> {
        self.inner.iter()
    }
}


