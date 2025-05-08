use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::fs::{File, OpenOptions, create_dir_all};
use std::path::{Path, PathBuf};

use crate::observability::telemetry::TelemetryClient;
use crate::observability::metrics::ObservabilityConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
    
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => LogLevel::Trace,
            1 => LogLevel::Debug,
            2 => LogLevel::Info,
            3 => LogLevel::Warn,
            4 => LogLevel::Error,
            5 => LogLevel::Fatal,
            _ => LogLevel::Info,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub module: String,
    pub thread_id: String,
    pub context: HashMap<String, String>,
}

pub struct Logger {
    min_level: LogLevel,
    log_file: Option<Arc<Mutex<File>>>,
    log_file_path: Option<PathBuf>,
    console_output: bool,
    telemetry_enabled: bool,
    max_file_size: u64,
    max_log_files: u32,
}

impl Logger {
    pub fn new(config: &ObservabilityConfig) -> Self {
        let min_level = config.min_log_level
            .map(LogLevel::from_u8)
            .unwrap_or(LogLevel::Info);
        
        let console_output = config.console_logging.unwrap_or(true);
        let telemetry_enabled = config.telemetry_enabled.unwrap_or(false) && 
                               config.log_telemetry.unwrap_or(false);
        
        let (log_file, log_file_path) = if let Some(log_path) = &config.log_file_path {
            // Create the directory if it doesn't exist
            if let Some(parent) = Path::new(log_path).parent() {
                let _ = create_dir_all(parent);
            }
            
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .ok();
                
            (file.map(|f| Arc::new(Mutex::new(f))), Some(PathBuf::from(log_path)))
        } else {
            (None, None)
        };
        
        Self {
            min_level,
            log_file,
            log_file_path,
            console_output,
            telemetry_enabled,
            max_file_size: 10 * 1024 * 1024, // 10 MB
            max_log_files: 5,
        }
    }
    
    pub fn log(&self, level: LogLevel, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        if level < self.min_level {
            return;
        }
        
        let thread_id = format!("{:?}", std::thread::current().id());
        
        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            message: message.to_string(),
            module: module.to_string(),
            thread_id,
            context: context.unwrap_or_default(),
        };
        
        // Log to console if enabled
        if self.console_output {
            self.log_to_console(&entry);
        }
        
        // Log to file if configured
        if let Some(file) = &self.log_file {
            self.log_to_file(&entry, file);
        }
        
        // Send to telemetry if enabled
        if self.telemetry_enabled {
            // In a real implementation, we would have:
            // let telemetry_client = TelemetryClient::get_instance();
            // telemetry_client.send_log(&entry);
        }
    }
    
    fn log_to_console(&self, entry: &LogEntry) {
        let formatted = format!(
            "[{}] [{}] [{}] {}: {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            entry.level.as_str(),
            entry.module,
            entry.thread_id,
            entry.message
        );
        
        match entry.level {
            LogLevel::Error | LogLevel::Fatal => eprintln!("{}", formatted),
            _ => println!("{}", formatted),
        }
    }
    
    fn log_to_file(&self, entry: &LogEntry, file: &Arc<Mutex<File>>) {
        if let Ok(serialized) = serde_json::to_string(entry) {
            if let Ok(mut file) = file.lock() {
                let _ = writeln!(file, "{}", serialized);
                
                // Check if rotation is needed
                if let Some(path) = &self.log_file_path {
                    // Get file size
                    if let Ok(metadata) = file.metadata() {
                        if metadata.len() > self.max_file_size {
                            // Rotation needed
                            drop(file); // Release the lock before rotation
                            self.rotate_log_file(path);
                        }
                    }
                }
            }
        }
    }
    
    fn rotate_log_file(&self, path: &Path) {
        // Implement log rotation
        for i in (1..self.max_log_files).rev() {
            let src = path.with_extension(format!("log.{}", i));
            let dst = path.with_extension(format!("log.{}", i + 1));
            if src.exists() {
                let _ = std::fs::rename(src, dst);
            }
        }
        
        let src = path;
        let dst = path.with_extension("log.1");
        let _ = std::fs::rename(src, dst);
        
        // Create a new log file
        if let Some(parent) = path.parent() {
            let _ = create_dir_all(parent);
        }
        
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path) {
            
            let mut log_file = self.log_file.as_ref().unwrap().lock().unwrap();
            *log_file = file;
        }
    }
    
    pub fn trace(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Trace, module, message, context);
    }
    
    pub fn debug(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Debug, module, message, context);
    }
    
    pub fn info(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Info, module, message, context);
    }
    
    pub fn warn(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Warn, module, message, context);
    }
    
    pub fn error(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Error, module, message, context);
    }
    
    pub fn fatal(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Fatal, module, message, context);
    }
    
    // Get recent logs for the UI
    pub fn get_recent_logs(&self, count: usize) -> Vec<LogEntry> {
        let mut logs = Vec::new();
        
        if let Some(path) = &self.log_file_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines().rev().take(count) {
                    if let Ok(entry) = serde_json::from_str::<LogEntry>(line) {
                        logs.push(entry);
                    }
                }
            }
        }
        
        logs.reverse();
        logs
    }
}

// Global logger instance
lazy_static::lazy_static! {
    pub static ref LOGGER: Arc<Mutex<Option<Logger>>> = Arc::new(Mutex::new(None));
}

// Initialize logger
pub fn init_logger(config: &ObservabilityConfig) {
    let logger = Logger::new(config);
    let mut global_logger = LOGGER.lock().unwrap();
    *global_logger = Some(logger);
}

// Logger macros for easy use
#[macro_export]
macro_rules! log_trace {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = crate::observability::logging::LOGGER.lock().unwrap().as_ref() {
            logger.trace($module, $message, $($context)?);
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = crate::observability::logging::LOGGER.lock().unwrap().as_ref() {
            logger.debug($module, $message, $($context)?);
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = crate::observability::logging::LOGGER.lock().unwrap().as_ref() {
            logger.info($module, $message, $($context)?);
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = crate::observability::logging::LOGGER.lock().unwrap().as_ref() {
            logger.warn($module, $message, $($context)?);
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = crate::observability::logging::LOGGER.lock().unwrap().as_ref() {
            logger.error($module, $message, $($context)?);
        }
    };
}

#[macro_export]
macro_rules! log_fatal {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = crate::observability::logging::LOGGER.lock().unwrap().as_ref() {
            logger.fatal($module, $message, $($context)?);
        }
    };
}

// Tauri commands for the UI
#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_recent_logs(count: usize) -> Vec<LogEntry> {
    if let Some(logger) = LOGGER.lock().unwrap().as_ref() {
        logger.get_recent_logs(count)
    } else {
        Vec::new()
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn export_logs(path: String) -> Result<(), String> {
    if let Some(logger) = LOGGER.lock().unwrap().as_ref() {
        if let Some(log_path) = &logger.log_file_path {
            if let Ok(content) = std::fs::read_to_string(log_path) {
                if let Err(err) = std::fs::write(&path, content) {
                    return Err(format!("Failed to write logs to {}: {}", path, err));
                }
                return Ok(());
            }
        }
        Err("No log file configured".to_string())
    } else {
        Err("Logger not initialized".to_string())
    }
}