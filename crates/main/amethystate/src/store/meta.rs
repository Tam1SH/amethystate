use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PrefixMeta {
    pub version: u32,
    pub hash: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StoredFieldEntry {
    pub name: String,
    pub type_name: String,
    pub type_hash: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SchemaSnapshot {
    pub version: u32,
    pub struct_name: Option<String>,
    pub schema_hash: u32,
    pub fields: Vec<StoredFieldEntry>,
}
