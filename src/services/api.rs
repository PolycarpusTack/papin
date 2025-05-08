use crate::utils::config;
use log::{debug, error, info, warn};
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Service for handling HTTP API requests
pub struct ApiService {
    /// HTTP client
    client: Client,
    
    /// Base API URL
    base_url: String,
    
    /// API key
    api_key: String,
}

/// API error type
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("HTTP error {0}: {1}")]
    HttpError(u16, String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Rate limit error: {0}")]
    RateLimitError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ApiService {
    /// Create a new API service
    pub fn new() -> Self {
        // Load configuration
        let config = config::get_config();
        let config_guard = config.lock().unwrap();
        
        let base_url = config_guard
            .get_string("api.base_url")
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());
        
        let api_key = config_guard
            .get_string("api.key")
            .unwrap_or_else(|| String::new());
        
        // Create HTTP client
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap();
        
        Self {
            client,
            base_url,
            api_key,
        }
    }
    
    /// Set API key
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }
    
    /// Set base URL
    pub fn set_base_url(&mut self, base_url: String) {
        self.base_url = base_url;
    }
    
    /// Make a GET request
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, path);
        
        let response = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        self.process_response(response).await
    }
    
    /// Make a POST request
    pub async fn post<T: DeserializeOwned, D: Serialize>(
        &self,
        path: &str,
        data: &D,
    ) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, path);
        
        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(data)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;
        
        self.process_response(response).await
    }
    
    /// Process the API response
    async fn process_response<T: DeserializeOwned>(&self, response: Response) -> Result<T, ApiError> {
        match response.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED => {
                // Parse response body
                response
                    .json::<T>()
                    .await
                    .map_err(|e| ApiError::SerializationError(e.to_string()))
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                let error_text = response.text().await.unwrap_or_default();
                Err(ApiError::AuthError(error_text))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let error_text = response.text().await.unwrap_or_default();
                Err(ApiError::RateLimitError(error_text))
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(ApiError::HttpError(status.as_u16(), error_text))
            }
        }
    }
}

/// Global API service instance
static API_SERVICE: once_cell::sync::OnceCell<ApiService> = once_cell::sync::OnceCell::new();

/// Get the global API service instance
pub fn get_api_service() -> &'static ApiService {
    API_SERVICE.get_or_init(|| ApiService::new())
}
