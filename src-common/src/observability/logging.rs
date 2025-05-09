use chrono::{DateTime, Utc, NaiveDate, Local};
use colored::Colorize;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::io::{self, Write};
use std::fs::{self, File, OpenOptions, create_dir_all};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::ops::Add;
use uuid::Uuid;

use crate::observability::telemetry::TelemetryClient;
use crate::observability::metrics::ObservabilityConfig;
use crate::error::Result;

// Define log levels
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
    
    pub fn from_str(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "TRACE" => LogLevel::Trace,
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARN" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            "FATAL" => LogLevel::Fatal,
            _ => LogLevel::Info,
        }
    }
    
    pub fn color_string(&self, text: &str) -> colored::ColoredString {
        match self {
            LogLevel::Trace => text.bright_black(),
            LogLevel::Debug => text.bright_blue(),
            LogLevel::Info => text.green(),
            LogLevel::Warn => text.yellow(),
            LogLevel::Error => text.red(),
            LogLevel::Fatal => text.bright_red().bold(),
        }
    }
}

// Define log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub module: String,
    pub thread_id: String,
    pub session_id: String,
    pub context: HashMap<String, String>,
    pub stacktrace: Option<String>,
}

// Logger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    pub min_level: LogLevel,
    pub file_enabled: bool,
    pub file_path: Option<String>,
    pub console_enabled: bool,
    pub console_format: LogFormat,
    pub colored_output: bool,
    pub max_file_size: u64,
    pub max_log_files: u32,
    pub telemetry_enabled: bool,
    pub include_context: bool,
    pub rotate_daily: bool,
    pub log_to_remote: bool,
    pub remote_url: Option<String>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            file_enabled: true,
            file_path: None,
            console_enabled: true,
            console_format: LogFormat::Detailed,
            colored_output: true,
            max_file_size: 10 * 1024 * 1024, // 10 MB
            max_log_files: 5,
            telemetry_enabled: false,
            include_context: true,
            rotate_daily: true,
            log_to_remote: false,
            remote_url: None,
        }
    }
}

// Log format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    Simple,     // [INFO] Message
    Standard,   // [2023-05-01 10:30:45] [INFO] [module] Message
    Detailed,   // [2023-05-01 10:30:45.123] [INFO] [module] [thread-id] Message
    Json,       // {"timestamp": "...", "level": "INFO", ...}
}

/// In-memory log buffer for UI display and analytics
#[derive(Debug, Clone, Default)]
pub struct LogBuffer {
    entries: Vec<LogEntry>,
    max_size: usize,
}

impl LogBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_size),
            max_size,
        }
    }
    
    pub fn add(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.max_size {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }
    
    pub fn get_recent(&self, count: usize) -> Vec<LogEntry> {
        let start = self.entries.len().saturating_sub(count);
        self.entries[start..].to_vec()
    }
    
    pub fn get_all(&self) -> Vec<LogEntry> {
        self.entries.clone()
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    pub fn filter(&self, level: Option<LogLevel>, module: Option<&str>, contains: Option<&str>) -> Vec<LogEntry> {
        self.entries.iter()
            .filter(|entry| {
                if let Some(min_level) = level {
                    if entry.level < min_level {
                        return false;
                    }
                }
                
                if let Some(mod_pattern) = module {
                    if !entry.module.contains(mod_pattern) {
                        return false;
                    }
                }
                
                if let Some(text) = contains {
                    if !entry.message.contains(text) && 
                       !entry.context.values().any(|v| v.contains(text)) {
                        return false;
                    }
                }
                
                true
            })
            .cloned()
            .collect()
    }
}

// Main Logger struct
pub struct Logger {
    config: RwLock<LoggerConfig>,
    log_file: Option<Arc<RwLock<File>>>,
    log_file_path: Option<PathBuf>,
    in_memory_buffer: RwLock<LogBuffer>,
    session_id: String,
    telemetry_client: Option<Arc<TelemetryClient>>,
    last_rotation_date: RwLock<NaiveDate>,
}

impl Logger {
    pub fn new(config: LoggerConfig, telemetry_client: Option<Arc<TelemetryClient>>) -> Self {
        let (log_file, log_file_path) = if config.file_enabled {
            if let Some(log_path) = &config.file_path {
                // Create the directory if it doesn't exist
                if let Some(parent) = Path::new(log_path).parent() {
                    let _ = create_dir_all(parent);
                }
                
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_path)
                    .ok();
                    
                (file.map(|f| Arc::new(RwLock::new(f))), Some(PathBuf::from(log_path)))
            } else {
                let default_path = Self::default_log_path();
                // Create the directory if it doesn't exist
                if let Some(parent) = default_path.parent() {
                    let _ = create_dir_all(parent);
                }
                
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&default_path)
                    .ok();
                    
                (file.map(|f| Arc::new(RwLock::new(f))), Some(default_path))
            }
        } else {
            (None, None)
        };
        
        Self {
            config: RwLock::new(config),
            log_file,
            log_file_path,
            in_memory_buffer: RwLock::new(LogBuffer::new(1000)), // Keep last 1000 log entries in memory
            session_id: Uuid::new_v4().to_string(),
            telemetry_client,
            last_rotation_date: RwLock::new(Local::now().date_naive()),
        }
    }
    
    fn default_log_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        
        // Platform-specific log directory
        #[cfg(target_os = "windows")]
        {
            path.push("AppData");
            path.push("Roaming");
            path.push("MCP");
            path.push("logs");
        }
        
        #[cfg(target_os = "macos")]
        {
            path.push("Library");
            path.push("Logs");
            path.push("MCP");
        }
        
        #[cfg(target_os = "linux")]
        {
            path.push(".local");
            path.push("share");
            path.push("mcp");
            path.push("logs");
        }
        
        // Ensure directory exists
        if !path.exists() {
            let _ = create_dir_all(&path);
        }
        
        // Add timestamp to log file name
        let now = Local::now();
        let filename = format!("mcp_{}.log", now.format("%Y-%m-%d"));
        path.push(filename);
        
        path
    }
    
    pub fn log(&self, level: LogLevel, module: &str, message: &str, context: Option<HashMap<String, String>>, stacktrace: Option<String>) {
        // Check log level
        if level < self.config.read().unwrap().min_level {
            return;
        }
        
        // Check for daily rotation if enabled
        if self.config.read().unwrap().rotate_daily {
            let today = Local::now().date_naive();
            let mut last_rotation = self.last_rotation_date.write().unwrap();
            
            if today > *last_rotation {
                if let Some(path) = &self.log_file_path {
                    self.rotate_log_file_daily(path, today);
                    *last_rotation = today;
                }
            }
        }
        
        let thread_id = format!("{:?}", std::thread::current().id());
        
        let entry = LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            message: message.to_string(),
            module: module.to_string(),
            thread_id,
            session_id: self.session_id.clone(),
            context: context.unwrap_or_default(),
            stacktrace,
        };
        
        // Log to console if enabled
        if self.config.read().unwrap().console_enabled {
            self.log_to_console(&entry);
        }
        
        // Log to file if configured
        if self.config.read().unwrap().file_enabled {
            if let Some(file) = &self.log_file {
                self.log_to_file(&entry, file);
            }
        }
        
        // Add to in-memory buffer
        {
            let mut buffer = self.in_memory_buffer.write().unwrap();
            buffer.add(entry.clone());
        }
        
        // Send to telemetry if enabled
        if self.config.read().unwrap().telemetry_enabled {
            if let Some(client) = &self.telemetry_client {
                if entry.level >= LogLevel::Error {
                    let _ = client.send_log(&entry);
                }
            }
        }
        
        // Send to remote logging service if enabled
        if self.config.read().unwrap().log_to_remote {
            if let Some(url) = &self.config.read().unwrap().remote_url {
                self.log_to_remote(&entry, url);
            }
        }
    }
    
    fn log_to_console(&self, entry: &LogEntry) {
        let config = self.config.read().unwrap();
        
        let formatted = match config.console_format {
            LogFormat::Simple => {
                format!("[{}] {}", entry.level.as_str(), entry.message)
            },
            LogFormat::Standard => {
                format!(
                    "[{}] [{}] [{}] {}",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    entry.level.as_str(),
                    entry.module,
                    entry.message
                )
            },
            LogFormat::Detailed => {
                let context_str = if config.include_context && !entry.context.is_empty() {
                    let ctx_pairs: Vec<String> = entry.context.iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect();
                    format!(" {{{}}}", ctx_pairs.join(", "))
                } else {
                    String::new()
                };
                
                format!(
                    "[{}] [{}] [{}] [{}] {}{}",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                    entry.level.as_str(),
                    entry.module,
                    entry.thread_id,
                    entry.message,
                    context_str
                )
            },
            LogFormat::Json => {
                serde_json::to_string(entry).unwrap_or_else(|_| format!("[ERROR] Failed to serialize log entry: {:?}", entry))
            },
        };
        
        let output = if config.colored_output {
            entry.level.color_string(&formatted).to_string()
        } else {
            formatted
        };
        
        match entry.level {
            LogLevel::Error | LogLevel::Fatal => eprintln!("{}", output),
            _ => println!("{}", output),
        }
    }
    
    fn log_to_file(&self, entry: &LogEntry, file: &Arc<RwLock<File>>) {
        // Format log entry as JSON for file
        if let Ok(serialized) = serde_json::to_string(entry) {
            if let Ok(mut file) = file.write() {
                let _ = writeln!(file, "{}", serialized);
                let _ = file.flush();
                
                // Check if rotation is needed
                if let Some(path) = &self.log_file_path {
                    // Get file size
                    if let Ok(metadata) = file.metadata() {
                        if metadata.len() > self.config.read().unwrap().max_file_size {
                            // Release the lock before rotation
                            drop(file);
                            
                            // Rotation needed
                            self.rotate_log_file_size(path);
                        }
                    }
                }
            }
        }
    }
    
    fn log_to_remote(&self, entry: &LogEntry, url: &str) {
        // In a real implementation, this would send logs to a remote logging service
        // This is a placeholder for demonstration purposes
        #[cfg(feature = "remote_logging")]
        {
            // Create a client
            let client = reqwest::blocking::Client::new();
            
            // Send the log to the remote service
            let _ = client.post(url)
                .json(entry)
                .send();
        }
    }
    
    fn rotate_log_file_size(&self, path: &Path) {
        let max_files = self.config.read().unwrap().max_log_files;
        
        // Implement log rotation by size
        for i in (1..max_files).rev() {
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
            
            if let Some(log_file) = &self.log_file {
                let mut guard = log_file.write().unwrap();
                *guard = file;
            }
        }
    }
    
    fn rotate_log_file_daily(&self, path: &Path, today: NaiveDate) {
        // Create new file name with date
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = path.extension().unwrap_or_default().to_string_lossy();
        
        // Archive the old log file with date
        let archive_name = format!("{}_{}.{}", 
                                  stem,
                                  self.last_rotation_date.read().unwrap().format("%Y-%m-%d"),
                                  ext);
        let archive_path = parent.join(archive_name);
        
        // Rename the current log file to the archive name
        if path.exists() {
            let _ = std::fs::rename(path, archive_path);
        }
        
        // Create a new log file
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path) {
            
            if let Some(log_file) = &self.log_file {
                let mut guard = log_file.write().unwrap();
                *guard = file;
            }
        }
        
        // Clean up old logs
        self.clean_old_logs(parent);
    }
    
    fn clean_old_logs(&self, dir: &Path) {
        let max_files = self.config.read().unwrap().max_log_files as usize;
        
        // Get all log files in directory
        if let Ok(entries) = fs::read_dir(dir) {
            let mut log_files: Vec<(PathBuf, NaiveDate)> = entries
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if path.extension()?.to_string_lossy() == "log" {
                        let filename = path.file_name()?.to_string_lossy();
                        if let Some(date_str) = filename.split('_').nth(1) {
                            if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                return Some((path, date));
                            }
                        }
                    }
                    None
                })
                .collect();
            
            // Sort by date (newest first)
            log_files.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Remove oldest files if we have too many
            if log_files.len() > max_files {
                for (path, _) in log_files.iter().skip(max_files) {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }
    
    pub fn trace(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Trace, module, message, context, None);
    }
    
    pub fn debug(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Debug, module, message, context, None);
    }
    
    pub fn info(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Info, module, message, context, None);
    }
    
    pub fn warn(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Warn, module, message, context, None);
    }
    
    pub fn error(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        // Capture stack trace for errors
        let stacktrace = backtrace::Backtrace::capture().to_string();
        self.log(LogLevel::Error, module, message, context, Some(stacktrace));
    }
    
    pub fn fatal(&self, module: &str, message: &str, context: Option<HashMap<String, String>>) {
        // Capture stack trace for fatal errors
        let stacktrace = backtrace::Backtrace::capture().to_string();
        self.log(LogLevel::Fatal, module, message, context, Some(stacktrace));
    }
    
    // Get recent logs for the UI
    pub fn get_recent_logs(&self, count: usize) -> Vec<LogEntry> {
        let buffer = self.in_memory_buffer.read().unwrap();
        buffer.get_recent(count)
    }
    
    // Search logs with filters
    pub fn search_logs(&self, level: Option<LogLevel>, module: Option<&str>, contains: Option<&str>) -> Vec<LogEntry> {
        let buffer = self.in_memory_buffer.read().unwrap();
        buffer.filter(level, module, contains)
    }
    
    // Export logs to a file
    pub fn export_logs(&self, path: &str, format: ExportFormat, filters: Option<LogFilters>) -> Result<()> {
        let buffer = self.in_memory_buffer.read().unwrap();
        
        // Apply filters if provided
        let logs = if let Some(filters) = filters {
            buffer.filter(filters.min_level, filters.module.as_deref(), filters.contains.as_deref())
        } else {
            buffer.get_all()
        };
        
        // Format logs based on export format
        let content = match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&logs)?
            },
            ExportFormat::Csv => {
                let mut csv_content = String::from("timestamp,level,module,message\n");
                for entry in &logs {
                    csv_content.push_str(&format!(
                        "{},{},{},{}\n",
                        entry.timestamp.to_rfc3339(),
                        entry.level.as_str(),
                        entry.module,
                        entry.message.replace(',', "\\,") // Escape commas in message
                    ));
                }
                csv_content
            },
            ExportFormat::Text => {
                let mut text_content = String::new();
                for entry in &logs {
                    text_content.push_str(&format!(
                        "[{}] [{}] [{}] {}\n",
                        entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        entry.level.as_str(),
                        entry.module,
                        entry.message
                    ));
                }
                text_content
            },
            ExportFormat::Html => {
                let mut html_content = String::from(
                    "<!DOCTYPE html>\n<html>\n<head>\n<style>\n\
                     table { border-collapse: collapse; width: 100%; }\n\
                     th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n\
                     tr:nth-child(even) { background-color: #f2f2f2; }\n\
                     th { background-color: #4CAF50; color: white; }\n\
                     .TRACE { color: #888; }\n\
                     .DEBUG { color: #00f; }\n\
                     .INFO { color: #080; }\n\
                     .WARN { color: #880; }\n\
                     .ERROR { color: #f00; }\n\
                     .FATAL { color: #f00; font-weight: bold; }\n\
                     </style>\n</head>\n<body>\n<h1>Log Export</h1>\n\
                     <table>\n<tr><th>Timestamp</th><th>Level</th><th>Module</th><th>Message</th></tr>\n"
                );
                
                for entry in &logs {
                    html_content.push_str(&format!(
                        "<tr><td>{}</td><td class=\"{}\">{}</td><td>{}</td><td>{}</td></tr>\n",
                        entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        entry.level.as_str(),
                        entry.level.as_str(),
                        entry.module,
                        entry.message.replace('<', "&lt;").replace('>', "&gt;") // Escape HTML tags
                    ));
                }
                
                html_content.push_str("</table>\n</body>\n</html>");
                html_content
            },
        };
        
        // Write to file
        fs::write(path, content)?;
        
        Ok(())
    }
    
    // Update logger configuration
    pub fn update_config(&self, new_config: LoggerConfig) -> Result<()> {
        let old_config = self.config.read().unwrap().clone();
        
        // Check if file path changed or file logging was enabled
        let file_path_changed = match (old_config.file_path.as_ref(), new_config.file_path.as_ref()) {
            (Some(old), Some(new)) => old != new,
            (None, Some(_)) => true,
            (Some(_), None) => true,
            (None, None) => false,
        };
        
        let file_enabled_changed = old_config.file_enabled != new_config.file_enabled;
        
        // Update config
        *self.config.write().unwrap() = new_config.clone();
        
        // Reinitialize file logging if needed
        if (file_path_changed || file_enabled_changed) && new_config.file_enabled {
            if let Some(log_path) = &new_config.file_path {
                // Create the directory if it doesn't exist
                if let Some(parent) = Path::new(log_path).parent() {
                    let _ = create_dir_all(parent);
                }
                
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_path)?;
                    
                self.log_file = Some(Arc::new(RwLock::new(file)));
                self.log_file_path = Some(PathBuf::from(log_path));
            }
        } else if !new_config.file_enabled {
            self.log_file = None;
            self.log_file_path = None;
        }
        
        Ok(())
    }
}

// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Text,
    Html,
}

// Log search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilters {
    pub min_level: Option<LogLevel>,
    pub module: Option<String>,
    pub contains: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

// Global logger instance
lazy_static::lazy_static! {
    pub static ref LOGGER: Arc<RwLock<Option<Logger>>> = Arc::new(RwLock::new(None));
}

// Initialize logger from config
pub fn init_logger(config: &ObservabilityConfig, telemetry_client: Option<Arc<TelemetryClient>>) {
    let logger_config = LoggerConfig {
        min_level: config.min_log_level.map(LogLevel::from_u8).unwrap_or(LogLevel::Info),
        file_enabled: true,
        file_path: config.log_file_path.clone(),
        console_enabled: config.console_logging.unwrap_or(true),
        console_format: LogFormat::Detailed,
        colored_output: true,
        max_file_size: 10 * 1024 * 1024, // 10 MB
        max_log_files: 5,
        telemetry_enabled: config.telemetry_enabled.unwrap_or(false) && config.log_telemetry.unwrap_or(false),
        include_context: true,
        rotate_daily: true,
        log_to_remote: false,
        remote_url: None,
    };
    
    let logger = Logger::new(logger_config, telemetry_client);
    
    // Initialize global logger
    *LOGGER.write().unwrap() = Some(logger);
    
    // Log initialization
    log_info!("logger", "Logger initialized");
}

// Logger macros for easy use
#[macro_export]
macro_rules! log_trace {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = $crate::observability::logging::LOGGER.read().unwrap().as_ref() {
            logger.trace($module, $message, $($context)?);
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = $crate::observability::logging::LOGGER.read().unwrap().as_ref() {
            logger.debug($module, $message, $($context)?);
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = $crate::observability::logging::LOGGER.read().unwrap().as_ref() {
            logger.info($module, $message, $($context)?);
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = $crate::observability::logging::LOGGER.read().unwrap().as_ref() {
            logger.warn($module, $message, $($context)?);
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = $crate::observability::logging::LOGGER.read().unwrap().as_ref() {
            logger.error($module, $message, $($context)?);
        }
    };
}

#[macro_export]
macro_rules! log_fatal {
    ($module:expr, $message:expr $(, $context:expr)?) => {
        if let Some(logger) = $crate::observability::logging::LOGGER.read().unwrap().as_ref() {
            logger.fatal($module, $message, $($context)?);
        }
    };
}

// Context builder for structured logging
#[macro_export]
macro_rules! log_context {
    ($($key:expr => $value:expr),*) => {
        {
            let mut context = std::collections::HashMap::new();
            $(
                context.insert($key.to_string(), $value.to_string());
            )*
            Some(context)
        }
    };
}

// Tauri commands for the UI
#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_recent_logs(count: usize) -> Vec<LogEntry> {
    if let Some(logger) = LOGGER.read().unwrap().as_ref() {
        logger.get_recent_logs(count)
    } else {
        Vec::new()
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn search_logs(level: Option<String>, module: Option<String>, contains: Option<String>) -> Vec<LogEntry> {
    if let Some(logger) = LOGGER.read().unwrap().as_ref() {
        logger.search_logs(
            level.map(|l| LogLevel::from_str(&l)),
            module.as_deref(),
            contains.as_deref(),
        )
    } else {
        Vec::new()
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn export_logs(path: String, format: String, filters: Option<LogFilters>) -> Result<()> {
    if let Some(logger) = LOGGER.read().unwrap().as_ref() {
        let export_format = match format.to_lowercase().as_str() {
            "json" => ExportFormat::Json,
            "csv" => ExportFormat::Csv,
            "text" => ExportFormat::Text,
            "html" => ExportFormat::Html,
            _ => ExportFormat::Json,
        };
        
        logger.export_logs(&path, export_format, filters)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Logger not initialized").into())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn update_logger_config(config: LoggerConfig) -> Result<()> {
    if let Some(logger) = LOGGER.read().unwrap().as_ref() {
        logger.update_config(config)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Logger not initialized").into())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub fn get_logger_config() -> Option<LoggerConfig> {
    if let Some(logger) = LOGGER.read().unwrap().as_ref() {
        Some(logger.config.read().unwrap().clone())
    } else {
        None
    }
}