use chrono::Utc;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Level {
    Trace = 0,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl Level {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "TRACE" => Level::Trace,
            "DEBUG" => Level::Debug,
            "INFO" => Level::Info,
            "WARN" => Level::Warn,
            "ERROR" => Level::Error,
            "FATAL" => Level::Fatal,
            _ => Level::Debug,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
            Level::Fatal => "FATAL",
        }
    }
}

pub struct LogSystem {
    seq: AtomicU64,
    min_level: Level,
    log_file: Mutex<File>,
    path: PathBuf,
}

impl LogSystem {
    pub fn new(session_id: &str, min_level: Level) -> std::io::Result<Arc<Self>> {
        let mut path = PathBuf::from("debugger/logs/active");
        std::fs::create_dir_all(&path)?;
        path.push(format!("{}.log", session_id));
        
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        
        Ok(Arc::new(Self {
            seq: AtomicU64::new(0),
            min_level,
            log_file: Mutex::new(file),
            path,
        }))
    }
    
    pub fn log(&self, level: Level, module: &str, message: String) {
        if level < self.min_level {
            return;
        }
        
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S.%3fZ");
        
        let log_line = format!("[{}] [SEQ:{}] [LEVEL:{}] [MODULE:{}] {}\n", now, seq, level.as_str(), module, message);
        
        if let Ok(mut file) = self.log_file.lock() {
            let _ = file.write_all(log_line.as_bytes());
        }
    }
    
    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
}
