use crate::models::Model;
use std::path::PathBuf;

/// Information about a local model
#[derive(Debug, Clone)]
pub struct LocalModelInfo {
    /// Model ID
    pub id: String,
    
    /// Model name
    pub name: String,
    
    /// Path to model file
    pub path: PathBuf,
    
    /// Number of parameters
    pub parameters: u64,
    
    /// Quantization type
    pub quantization: String,
    
    /// Context size (max tokens)
    pub context_size: usize,
    
    /// Whether the model is downloaded
    pub is_downloaded: bool,
    
    /// URL to download the model
    pub download_url: Option<String>,
    
    /// Model metadata
    pub model: Model,
}
