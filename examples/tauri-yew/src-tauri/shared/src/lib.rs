use amethystate::AmeType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, AmeType)]
pub struct ProxyProfile {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub enabled: bool,
}

impl Default for ProxyProfile {
    fn default() -> Self {
        Self {
            name: "Default Proxy".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            enabled: false,
        }
    }
}