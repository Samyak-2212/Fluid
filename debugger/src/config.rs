use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct DebuggerConfig {
    pub debugger_http_port: u16,
    pub log_level: String,
    pub max_active_log_size_mb: u64,
    pub session_id_format: String,
}

impl DebuggerConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {:?}: {}", path, e))?;
        toml::from_str(&raw).map_err(|e| format!("parse error in {:?}: {}", path, e))
    }
    
    pub fn default_config() -> Self {
        Self {
            debugger_http_port: 8081,
            log_level: "DEBUG".to_string(),
            max_active_log_size_mb: 100,
            session_id_format: "timestamp".to_string(),
        }
    }
}
