use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfsConfig {
    pub bind_addr: String,
    pub port: u16,
}

impl Default for NfsConfig {
    fn default() -> Self {
        Self { bind_addr: "0.0.0.0".into(), port: 2049 }
    }
}
