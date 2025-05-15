// src/offline/checkpointing/mod.rs
// Enhanced to use platform-agnostic file operations

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::fs::{platform_fs, PlatformFsError, PathExt};

/// Checkpoint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Checkpoint ID
    pub id: String,
    /// Checkpoint name
    pub name: String,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Size in bytes
    pub size_bytes: usize,
    /// Compression ratio
    pub compression_ratio: f32,
    /// Number of items
    pub item_count: usize,
    /// Tags for organization
    pub tags: Vec<String>,
}

/// Checkpoint manager for saving and restoring conversation state
pub struct CheckpointManager {
    base_path: PathBuf,
    checkpoints: HashMap<String, CheckpointMetadata>,
    max_checkpoints: usize,
    compression_level: u32,
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new() -> Self {
        // Get default checkpoint directory
        let fs = platform_fs();
        let default_path = fs
            .cache_dir("Papin")
            .map(|p| p.join("checkpoints"))
            .unwrap_or_else(|_| PathBuf::from("checkpoints"));

        Self {
            base_path: default_path,
            checkpoints: HashMap::new(),
            max_checkpoints: 10,
            compression_level: 6,
        }
    }

    /// Set the base path for checkpoints
    pub fn with_base_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.base_path = path.as_ref().to_path_buf();
        self
    }

    /// Set the maximum number of checkpoints to keep
    pub fn with_max_checkpoints(mut self, max: usize) -> Self {
        self.max_checkpoints = max;
        self
    }

    /// Set the compression level (0-9)
    pub fn with_compression_level(mut self, level: u32) -> Self {
        self.compression_level = level.min(9);
        self
    }

    /// Initialize the checkpoint manager
    pub fn initialize(&mut self) -> Result<(), String> {
        // Create base directory if it doesn't exist
        let fs = platform_fs();
        if let Err(e) = fs.ensure_dir_exists(&self.base_path) {
            return Err(format!("Failed to create checkpoint directory: {}", e));
        }

        // Load existing checkpoints
        self.load_checkpoints()
    }

    /// Load existing checkpoints
    fn load_checkpoints(&mut self) -> Result<(), String> {
        let fs = platform_fs();
        if !fs.dir_exists(&self.base_path) {
            return Ok(());
        }

        // Read directory contents
        let entries = match std::fs::read_dir(&self.base_path) {
            Ok(entries) => entries,
            Err(e) => return Err(format!("Failed to read checkpoint directory: {}", e)),
        };

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                // Skip non-JSON files
                if !path.is_file() || path.extension().map_or(true, |ext| ext != "json") {
                    continue;
                }

                // Read metadata file
                match fs.read_to_string(&path) {
                    Ok(content) => {
                        if let Ok(metadata) = serde_json::from_str::<CheckpointMetadata>(&content) {
                            self.checkpoints.insert(metadata.id.clone(), metadata);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read checkpoint metadata file: {}", e);
                    }
                }
            }
        }

        info!("Loaded {} checkpoints", self.checkpoints.len());
        Ok(())
    }

    /// Save a checkpoint
    pub fn save_checkpoint<T: Serialize>(&self, name: &str, data: T) -> String {
        debug!("Saving checkpoint: {}", name);

        // Generate checkpoint ID
        let id = Uuid::new_v4().to_string();

        // Create checkpoint directory if it doesn't exist
        let fs = platform_fs();
        if let Err(e) = fs.ensure_dir_exists(&self.base_path) {
            error!("Failed to create checkpoint directory: {}", e);
            return id;
        }

        // Serialize data
        let serialized = match serde_json::to_vec(&data) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to serialize checkpoint data: {}", e);
                return id;
            }
        };

        // Compress data
        let uncompressed_size = serialized.len();
        let compressed = match compress(&serialized, self.compression_level) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to compress checkpoint data: {}", e);
                return id;
            }
        };

        let compressed_size = compressed.len();
        let compression_ratio = uncompressed_size as f32 / compressed_size as f32;

        debug!(
            "Checkpoint compression: {} bytes -> {} bytes (ratio: {:.2})",
            uncompressed_size, compressed_size, compression_ratio
        );

        // Create metadata
        let metadata = CheckpointMetadata {
            id: id.clone(),
            name: name.to_string(),
            created_at: Utc::now(),
            size_bytes: compressed_size,
            compression_ratio,
            item_count: count_items(&data),
            tags: vec![],
        };

        // Save metadata
        let metadata_path = self.base_path.join(format!("{}.json", id));
        if let Err(e) = save_json(&metadata_path, &metadata) {
            error!("Failed to save checkpoint metadata: {}", e);
            return id;
        }

        // Save data
        let data_path = self.base_path.join(format!("{}.bin", id));
        if let Err(e) = save_binary(&data_path, &compressed) {
            error!("Failed to save checkpoint data: {}", e);
            return id;
        }

        info!("Checkpoint saved: {} ({})", name, id);
        id
    }

    /// Load a checkpoint
    pub fn load_checkpoint<T: for<'de> Deserialize<'de>>(&self, id: &str) -> Option<T> {
        debug!("Loading checkpoint: {}", id);

        // Check if checkpoint exists
        if !self.checkpoints.contains_key(id) {
            warn!("Checkpoint not found: {}", id);
            return None;
        }

        // Load compressed data
        let data_path = self.base_path.join(format!("{}.bin", id));
        let fs = platform_fs();
        
        // Open file
        let file = match fs.open_file(&data_path, false) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open checkpoint data file: {}", e);
                return None;
            }
        };
        
        // Read data
        let mut compressed = Vec::new();
        let mut reader = std::io::BufReader::new(file);
        if let Err(e) = reader.read_to_end(&mut compressed) {
            error!("Failed to read checkpoint data: {}", e);
            return None;
        }

        // Decompress data
        let decompressed = match decompress(&compressed) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to decompress checkpoint data: {}", e);
                return None;
            }
        };

        // Deserialize data
        match serde_json::from_slice(&decompressed) {
            Ok(data) => {
                info!("Checkpoint loaded: {}", id);
                Some(data)
            }
            Err(e) => {
                error!("Failed to deserialize checkpoint data: {}", e);
                None
            }
        }
    }

    /// Delete a checkpoint
    pub fn delete_checkpoint(&mut self, id: &str) -> Result<(), String> {
        debug!("Deleting checkpoint: {}", id);

        // Check if checkpoint exists
        if !self.checkpoints.contains_key(id) {
            return Err(format!("Checkpoint not found: {}", id));
        }

        let fs = platform_fs();

        // Remove metadata file
        let metadata_path = self.base_path.join(format!("{}.json", id));
        if let Err(e) = fs.remove_file(&metadata_path) {
            warn!("Failed to delete checkpoint metadata file: {}", e);
        }

        // Remove data file
        let data_path = self.base_path.join(format!("{}.bin", id));
        if let Err(e) = fs.remove_file(&data_path) {
            warn!("Failed to delete checkpoint data file: {}", e);
        }

        // Remove from memory
        self.checkpoints.remove(id);

        info!("Checkpoint deleted: {}", id);
        Ok(())
    }

    /// List all checkpoints
    pub fn list_checkpoints(&self) -> Vec<CheckpointMetadata> {
        self.checkpoints.values().cloned().collect()
    }

    /// Get a specific checkpoint metadata
    pub fn get_checkpoint_metadata(&self, id: &str) -> Option<CheckpointMetadata> {
        self.checkpoints.get(id).cloned()
    }

    /// Clean up old checkpoints
    pub fn cleanup_old_checkpoints(&mut self) -> Result<usize, String> {
        if self.checkpoints.len() <= self.max_checkpoints {
            return Ok(0);
        }

        // Sort checkpoints by creation time
        let mut checkpoints: Vec<_> = self.checkpoints.values().cloned().collect();
        checkpoints.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        // Delete oldest checkpoints
        let to_delete = checkpoints.len() - self.max_checkpoints;
        let mut deleted = 0;

        for i in 0..to_delete {
            if let Err(e) = self.delete_checkpoint(&checkpoints[i].id) {
                warn!("Failed to delete checkpoint {}: {}", checkpoints[i].id, e);
            } else {
                deleted += 1;
            }
        }

        info!("Cleaned up {} old checkpoints", deleted);
        Ok(deleted)
    }
    
    /// Get the base path for checkpoints
    pub fn get_base_path(&self) -> PathBuf {
        self.base_path.clone()
    }
    
    /// Set the base path for checkpoints
    pub fn set_base_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let fs = platform_fs();
        let new_path = path.as_ref().to_path_buf();
        
        // Create the new directory if it doesn't exist
        if let Err(e) = fs.ensure_dir_exists(&new_path) {
            return Err(format!("Failed to create checkpoint directory: {}", e));
        }
        
        // Move existing checkpoint files if needed
        if !self.checkpoints.is_empty() && self.base_path != new_path {
            for (id, _) in &self.checkpoints {
                // Move metadata file
                let old_metadata_path = self.base_path.join(format!("{}.json", id));
                let new_metadata_path = new_path.join(format!("{}.json", id));
                
                if fs.file_exists(&old_metadata_path) {
                    if let Err(e) = fs.rename_file(&old_metadata_path, &new_metadata_path) {
                        warn!("Failed to move checkpoint metadata file: {}", e);
                    }
                }
                
                // Move data file
                let old_data_path = self.base_path.join(format!("{}.bin", id));
                let new_data_path = new_path.join(format!("{}.bin", id));
                
                if fs.file_exists(&old_data_path) {
                    if let Err(e) = fs.rename_file(&old_data_path, &new_data_path) {
                        warn!("Failed to move checkpoint data file: {}", e);
                    }
                }
            }
        }
        
        // Update base path
        self.base_path = new_path;
        
        Ok(())
    }
}

// Helper functions

/// Count the number of items in serializable data
fn count_items<T: Serialize>(data: &T) -> usize {
    // For HashMap, count the number of entries
    if let Ok(map) = serde_json::to_value(data) {
        if map.is_object() {
            return map.as_object().unwrap().len();
        } else if map.is_array() {
            return map.as_array().unwrap().len();
        }
    }

    1
}

/// Compress data using zstd
fn compress(data: &[u8], level: u32) -> Result<Vec<u8>, String> {
    let mut encoder = match zstd::Encoder::new(Vec::new(), level as i32) {
        Ok(encoder) => encoder,
        Err(e) => return Err(format!("Failed to create zstd encoder: {}", e)),
    };

    // Write data
    if let Err(e) = encoder.write_all(data) {
        return Err(format!("Failed to compress data: {}", e));
    }

    // Finish encoding
    match encoder.finish() {
        Ok(compressed) => Ok(compressed),
        Err(e) => Err(format!("Failed to finalize compression: {}", e)),
    }
}

/// Decompress data using zstd
fn decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = match zstd::Decoder::new(data) {
        Ok(decoder) => decoder,
        Err(e) => return Err(format!("Failed to create zstd decoder: {}", e)),
    };

    let mut decompressed = Vec::new();

    // Read decompressed data
    if let Err(e) = decoder.read_to_end(&mut decompressed) {
        return Err(format!("Failed to decompress data: {}", e));
    }

    Ok(decompressed)
}

/// Save JSON to a file
fn save_json<T: Serialize>(path: &Path, data: &T) -> Result<(), String> {
    let fs = platform_fs();
    let json = match serde_json::to_string_pretty(data) {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to serialize JSON: {}", e)),
    };
    
    match fs.write_string(path, &json) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write JSON: {}", e)),
    }
}

/// Save binary data to a file
fn save_binary(path: &Path, data: &[u8]) -> Result<(), String> {
    let fs = platform_fs();
    let mut file = match fs.create_file(path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to create file: {}", e)),
    };
    
    match file.write_all(data) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write binary data: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load_checkpoint() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = CheckpointManager::new()
            .with_base_path(temp_dir.path())
            .with_max_checkpoints(5)
            .with_compression_level(6);

        // Create test data
        let data: HashMap<String, String> = [
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
            ("key3".to_string(), "value3".to_string()),
        ]
        .iter()
        .cloned()
        .collect();

        // Save checkpoint
        let id = manager.save_checkpoint("test", &data);

        // Load checkpoint
        let loaded: Option<HashMap<String, String>> = manager.load_checkpoint(&id);

        assert!(loaded.is_some());
        let loaded = loaded.unwrap();

        assert_eq!(loaded.len(), data.len());
        assert_eq!(loaded.get("key1"), Some(&"value1".to_string()));
        assert_eq!(loaded.get("key2"), Some(&"value2".to_string()));
        assert_eq!(loaded.get("key3"), Some(&"value3".to_string()));
    }
    
    #[test]
    fn test_checkpoint_directory() {
        let manager = CheckpointManager::new();
        
        // Verify base path is set
        let base_path = manager.get_base_path();
        assert!(!base_path.to_string_lossy().is_empty());
        
        // Default should be in the cache directory
        let fs = platform_fs();
        let expected_prefix = fs.cache_dir("Papin").unwrap_or_else(|_| PathBuf::from(""));
        
        // On Windows, normalize the paths for case-insensitive comparison
        #[cfg(target_os = "windows")]
        {
            let base_lower = base_path.to_string_lossy().to_lowercase();
            let expected_lower = expected_prefix.to_string_lossy().to_lowercase();
            assert!(base_lower.contains(&expected_lower) || expected_lower.is_empty());
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // On Unix systems, check if the base path starts with the expected prefix
            if !expected_prefix.as_os_str().is_empty() {
                assert!(base_path.starts_with(&expected_prefix) || base_path.ends_with("checkpoints"));
            }
        }
    }
}
