pub mod bug_pool_reporter;
pub mod config;
pub mod event_bridge;
pub mod events;
pub mod http_server;
pub mod log_archiver;
pub mod log_system;

use std::sync::{Arc, Mutex};
use crate::config::DebuggerConfig;
use crate::http_server::{DebuggerHttpServer, Stats};
use crate::log_system::{Level, LogSystem};
use std::path::Path;

pub struct Debugger {
    pub logger: Arc<LogSystem>,
    pub stats: Arc<Mutex<Stats>>,
}

impl Debugger {
    pub fn new() -> std::io::Result<Self> {
        let config_path = Path::new("config/debugger.toml");
        let config = DebuggerConfig::load(config_path).unwrap_or_else(|_| DebuggerConfig::default_config());
        
        let session_id = if config.session_id_format == "uuid" {
            uuid::Uuid::new_v4().to_string()
        } else {
            chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string()
        };
        
        let min_level = Level::from_str(&config.log_level);
        let logger = LogSystem::new(&session_id, min_level)?;
        let stats = Arc::new(Mutex::new(Stats::default()));
        
        let server = DebuggerHttpServer::new(config.debugger_http_port, logger.clone(), stats.clone());
        server.run();
        
        Ok(Self { logger, stats })
    }
    
    pub fn wire_events<T: core::event_bus::EventBus + ?Sized>(&self, event_bus: &T) {
        event_bridge::wire(event_bus, self.logger.clone(), self.stats.clone());
    }
}
