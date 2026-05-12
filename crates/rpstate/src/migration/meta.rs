use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PrefixMeta {
    pub version: u32,
    pub hash: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StoredFieldEntry {
    pub name: String,
    pub type_name: String,
    pub type_hash: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SchemaSnapshot {
    pub version: u32,
    pub fields: Vec<StoredFieldEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiffEntry {
    pub timestamp: u64,
    pub old_hash: u64,
    pub new_hash: u64,
}
