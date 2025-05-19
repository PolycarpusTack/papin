// src-common/src/platform/fs.rs
// Platform-agnostic file system service

use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::env;
use std::fmt;

/// Enumeration of supported platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

/// Error type for platform-specific file operations
#[derive(Debug)]
pub enum PlatformFsError {
    IoError(io::Error),
    PathError(String),
    PermissionDenied(PathBuf),
    PlatformNotSupported(Platform),
    FileLocked(PathBuf),
    DirectoryNotFound(PathBuf),
}

impl fmt::Display for PlatformFsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "I/O error: {}", e),
            Self::PathError(msg) => write!(f, "Path error: {}", msg),
            Self::PermissionDenied(path) => write!(f, "Permission denied: {}", path.display()),
            Self::PlatformNotSupported(platform) => write!(f, "Platform not supported: {:?}", platform),
            Self::FileLocked(path) => write!(f, "File is locked: {}", path.display()),
            Self::DirectoryNotFound(path) => write!(f, "Directory not found: {}", path.display()),
        }
    }
}

impl std::error::Error for PlatformFsError {}

impl From<io::Error> for PlatformFsError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::PermissionDenied => {
                // Try to extract the path from the error message
                let message = error.to_string();
                if let Some(path_str) = message.split_whitespace().last() {
                    Self::PermissionDenied(PathBuf::from(path_str))
                } else {
                    Self::PermissionDenied(PathBuf::from("<unknown path>"))
                }
            }
            io::ErrorKind::NotFound => {
                // Try to extract the path from the error message
                let message = error.to_string();
                if let Some(path_str) = message.split_whitespace().last() {
                    Self::DirectoryNotFound(PathBuf::from(path_str))
                } else {
                    Self::DirectoryNotFound(PathBuf::from("<unknown path>"))
                }
            }
            _ => Self::IoError(error),
        }
    }
}

type Result<T> = std::result::Result<T, PlatformFsError>;

/// Service to handle platform-specific file system operations
pub struct PlatformFs {
    platform: Platform,
}

impl PlatformFs {
    /// Create a new PlatformFs instance
    pub fn new() -> Self {
        Self {
            platform: detect_platform(),
        }
    }

    /// Get the current platform
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// Get the application data directory for the current platform
    pub fn app_data_dir(&self, app_name: &str) -> Result<PathBuf> {
        match self.platform {
            Platform::Windows => {
                // Windows: %APPDATA%\{app_name}
                match env::var("APPDATA") {
                    Ok(appdata) => Ok(PathBuf::from(appdata).join(app_name)),
                    Err(_) => Err(PlatformFsError::PathError("APPDATA environment variable not found".to_string())),
                }
            }
            Platform::MacOS => {
                // macOS: ~/Library/Application Support/{app_name}
                match dirs::home_dir() {
                    Some(home) => Ok(home.join("Library/Application Support").join(app_name)),
                    None => Err(PlatformFsError::PathError("Home directory not found".to_string())),
                }
            }
            Platform::Linux => {
                // Linux: ~/.config/{app_name}
                match dirs::home_dir() {
                    Some(home) => Ok(home.join(".config").join(app_name)),
                    None => Err(PlatformFsError::PathError("Home directory not found".to_string())),
                }
            }
            Platform::Unknown => Err(PlatformFsError::PlatformNotSupported(Platform::Unknown)),
        }
    }

    /// Get the cache directory for the current platform
    pub fn cache_dir(&self, app_name: &str) -> Result<PathBuf> {
        match self.platform {
            Platform::Windows => {
                // Windows: %LOCALAPPDATA%\{app_name}\cache
                match env::var("LOCALAPPDATA") {
                    Ok(local_appdata) => Ok(PathBuf::from(local_appdata).join(app_name).join("cache")),
                    Err(_) => Err(PlatformFsError::PathError("LOCALAPPDATA environment variable not found".to_string())),
                }
            }
            Platform::MacOS => {
                // macOS: ~/Library/Caches/{app_name}
                match dirs::home_dir() {
                    Some(home) => Ok(home.join("Library/Caches").join(app_name)),
                    None => Err(PlatformFsError::PathError("Home directory not found".to_string())),
                }
            }
            Platform::Linux => {
                // Linux: ~/.cache/{app_name}
                match dirs::home_dir() {
                    Some(home) => Ok(home.join(".cache").join(app_name)),
                    None => Err(PlatformFsError::PathError("Home directory not found".to_string())),
                }
            }
            Platform::Unknown => Err(PlatformFsError::PlatformNotSupported(Platform::Unknown)),
        }
    }

    /// Get the logs directory for the current platform
    pub fn logs_dir(&self, app_name: &str) -> Result<PathBuf> {
        match self.platform {
            Platform::Windows => {
                // Windows: %LOCALAPPDATA%\{app_name}\logs
                match env::var("LOCALAPPDATA") {
                    Ok(local_appdata) => Ok(PathBuf::from(local_appdata).join(app_name).join("logs")),
                    Err(_) => Err(PlatformFsError::PathError("LOCALAPPDATA environment variable not found".to_string())),
                }
            }
            Platform::MacOS => {
                // macOS: ~/Library/Logs/{app_name}
                match dirs::home_dir() {
                    Some(home) => Ok(home.join("Library/Logs").join(app_name)),
                    None => Err(PlatformFsError::PathError("Home directory not found".to_string())),
                }
            }
            Platform::Linux => {
                // Linux: ~/.local/share/{app_name}/logs
                match dirs::home_dir() {
                    Some(home) => Ok(home.join(".local/share").join(app_name).join("logs")),
                    None => Err(PlatformFsError::PathError("Home directory not found".to_string())),
                }
            }
            Platform::Unknown => Err(PlatformFsError::PlatformNotSupported(Platform::Unknown)),
        }
    }

    /// Ensure a directory exists, creating it if necessary
    pub fn ensure_dir_exists(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| map_io_error_to_platform_error(e, self.platform, path))?;
        } else if !path.is_dir() {
            return Err(PlatformFsError::PathError(format!("{} exists but is not a directory", path.display())));
        }

        // Check if directory is writable
        let test_file_path = path.join(".write_test");
        match File::create(&test_file_path) {
            Ok(_) => {
                // Successfully created test file, clean it up
                let _ = fs::remove_file(test_file_path);
                Ok(())
            }
            Err(e) => Err(map_io_error_to_platform_error(e, self.platform, path)),
        }
    }

    /// Create a file with proper permissions for the current platform
    pub fn create_file(&self, path: &Path) -> Result<File> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            self.ensure_dir_exists(parent)?;
        }

        // Create file with platform-specific options
        let file = match self.platform {
            Platform::Windows => {
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)
            }
            Platform::MacOS | Platform::Linux => {
                let mut options = OpenOptions::new();
                options.write(true).create(true).truncate(true);
                
                // Set mode to 0o644 on Unix platforms
                #[cfg(unix)]
                use std::os::unix::fs::OpenOptionsExt;
                #[cfg(unix)]
                options.mode(0o644);
                
                options.open(path)
            }
            Platform::Unknown => {
                return Err(PlatformFsError::PlatformNotSupported(Platform::Unknown));
            }
        };

        file.map_err(|e| map_io_error_to_platform_error(e, self.platform, path))
    }

    /// Open a file with proper permissions for the current platform
    pub fn open_file(&self, path: &Path, write: bool) -> Result<File> {
        let file = match (self.platform, write) {
            (Platform::Windows, true) => {
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(path)
            }
            (Platform::Windows, false) => {
                OpenOptions::new()
                    .read(true)
                    .open(path)
            }
            (Platform::MacOS, true) | (Platform::Linux, true) => {
                let mut options = OpenOptions::new();
                options.read(true).write(true);
                
                options.open(path)
            }
            (Platform::MacOS, false) | (Platform::Linux, false) => {
                let mut options = OpenOptions::new();
                options.read(true);
                
                options.open(path)
            }
            (Platform::Unknown, _) => {
                return Err(PlatformFsError::PlatformNotSupported(Platform::Unknown));
            }
        };

        file.map_err(|e| map_io_error_to_platform_error(e, self.platform, path))
    }

    /// Read a file's contents to a string
    pub fn read_to_string(&self, path: &Path) -> Result<String> {
        let mut file = self.open_file(path, false)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| map_io_error_to_platform_error(e, self.platform, path))?;
        Ok(contents)
    }

    /// Write a string to a file
    pub fn write_string(&self, path: &Path, contents: &str) -> Result<()> {
        let mut file = self.create_file(path)?;
        file.write_all(contents.as_bytes())
            .map_err(|e| map_io_error_to_platform_error(e, self.platform, path))?;
        file.flush()
            .map_err(|e| map_io_error_to_platform_error(e, self.platform, path))
    }

    /// Safely rename a file, handling cross-device moves
    pub fn rename_file(&self, from: &Path, to: &Path) -> Result<()> {
        // Try the simple rename first
        match fs::rename(from, to) {
            Ok(_) => return Ok(()),
            Err(e) => {
                // On some platforms, rename fails if the source and destination are on different filesystems
                if e.kind() == io::ErrorKind::CrossesDevices {
                    // Fall back to copy and delete
                    fs::copy(from, to)
                        .map_err(|e| map_io_error_to_platform_error(e, self.platform, to))?;
                    fs::remove_file(from)
                        .map_err(|e| map_io_error_to_platform_error(e, self.platform, from))?;
                    Ok(())
                } else {
                    Err(map_io_error_to_platform_error(e, self.platform, from))
                }
            }
        }
    }

    /// Remove a file
    pub fn remove_file(&self, path: &Path) -> Result<()> {
        fs::remove_file(path)
            .map_err(|e| map_io_error_to_platform_error(e, self.platform, path))
    }

    /// Check if a file exists
    pub fn file_exists(&self, path: &Path) -> bool {
        path.exists() && path.is_file()
    }

    /// Check if a directory exists
    pub fn dir_exists(&self, path: &Path) -> bool {
        path.exists() && path.is_dir()
    }

    /// Join paths in a platform-agnostic way
    pub fn join_paths<P: AsRef<Path>, I: IntoIterator<Item = P>>(&self, base: &Path, components: I) -> PathBuf {
        components.into_iter().fold(base.to_path_buf(), |acc, component| {
            acc.join(component)
        })
    }

    /// Get a temporary directory
    pub fn temp_dir(&self) -> PathBuf {
        env::temp_dir()
    }

    /// Create a unique temporary file path
    pub fn temp_file_path(&self, prefix: &str, extension: &str) -> PathBuf {
        let mut path = self.temp_dir();
        let file_name = format!("{}_{}.{}", 
            prefix, 
            chrono::Utc::now().timestamp(),
            extension
        );
        path.push(file_name);
        path
    }
}

/// Detect current platform
fn detect_platform() -> Platform {
    #[cfg(target_os = "windows")]
    return Platform::Windows;
    
    #[cfg(target_os = "macos")]
    return Platform::MacOS;
    
    #[cfg(target_os = "linux")]
    return Platform::Linux;
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return Platform::Unknown;
}

/// Map IO errors to platform-specific errors
fn map_io_error_to_platform_error(error: io::Error, platform: Platform, path: &Path) -> PlatformFsError {
    match error.kind() {
        io::ErrorKind::PermissionDenied => {
            match platform {
                Platform::Windows => {
                    // Check if this might be due to file being in use
                    PlatformFsError::FileLocked(path.to_path_buf())
                }
                _ => PlatformFsError::PermissionDenied(path.to_path_buf()),
            }
        }
        io::ErrorKind::NotFound => PlatformFsError::DirectoryNotFound(path.to_path_buf()),
        _ => PlatformFsError::IoError(error),
    }
}

// Create a lazy static instance for global use
lazy_static::lazy_static! {
    pub static ref PLATFORM_FS: PlatformFs = PlatformFs::new();
}

// Helper function to get the global instance
pub fn platform_fs() -> &'static PlatformFs {
    &PLATFORM_FS
}

// Helper trait for path normalization
pub trait PathExt {
    /// Convert a path to a platform-agnostic representation
    fn normalize(&self) -> PathBuf;
}

impl PathExt for Path {
    fn normalize(&self) -> PathBuf {
        // This ensures paths are properly normalized based on the platform
        // It removes redundancies like ".." and "."
        match self.canonicalize() {
            Ok(path) => path,
            Err(_) => self.to_path_buf(),
        }
    }
}
