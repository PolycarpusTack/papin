use log::{debug, error, warn};
use reqwest::{Client, Response, StatusCode};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio_stream::Stream;
use futures_util::StreamExt;

/// Claude API response
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeResponse {
    /// Response ID
    pub id: String,
    
    /// Response type
    #[serde(rename = "type")]
    pub response_type: String,
    
    /// Response role
    pub role: String,
    
    /// Model used
    pub model: String,
    
    /// Stop reason
    pub stop_reason: Option<String>,
    
    /// Content parts
    pub content: Vec<Value>,
    
    /// Usage statistics
    pub usage: Value,
}

/// Claude API delta response (for streaming)
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeDeltaResponse {
    /// Response type
    #[serde(rename = "type")]
    pub response_type: String,
    
    /// Message ID
    pub message_id: String,
    
    /// Delta content
    pub delta: Value,
    
    /// Usage (only present in the final chunk)
    pub usage: Option<Value>,
}

/// Claude API error
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeError {
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,
    
    /// Error message
    pub message: String,
}

/// Claude API client
#[derive(Clone)]
pub struct ClaudeApi {
    /// HTTP client
    client: Client,
    
    /// API base URL
    base_url: String,
    
    /// API key
    api_key: String,
    
    /// API version
    api_version: String,
}

impl ClaudeApi {
    /// Create a new Claude API client
    pub fn new(api_key: String, base_url: &str) -> Self {
        // Create client with default settings
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self {
            client,
            base_url: base_url.to_string(),
            api_key,
            api_version: "2023-06-01".to_string(),
        }
    }
    
    /// Set API key
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }
    
    /// Set API version
    pub fn set_api_version(&mut self, api_version: String) {
        self.api_version = api_version;
    }
    
    /// Create default headers
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        
        // Add authentication header
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .unwrap_or_else(|_| HeaderValue::from_static("")),
        );
        
        // Add content type
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        
        // Add API version
        headers.insert(
            "anthropic-version",
            HeaderValue::from_str(&self.api_version)
                .unwrap_or_else(|_| HeaderValue::from_static("2023-06-01")),
        );
        
        headers
    }
    
    /// Create a new message
    pub async fn create_message(&self, body: &Value) -> Result<ClaudeResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/v1/messages", self.base_url);
        
        let response = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(body)
            .send()
            .await?;
        
        self.handle_response(response).await
    }
    
    /// Create a streaming message
    pub async fn create_message_stream(
        &self,
        body: &Value,
    ) -> Result<impl Stream<Item = Result<ClaudeDeltaResponse, Box<dyn std::error::Error + Send + Sync>>>, Box<dyn std::error::Error>> {
        let url = format!("{}/v1/messages", self.base_url);
        
        let response = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await?);
        }
        
        let stream = response.bytes_stream().map(|result| {
            result.map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send + Sync>)
                .and_then(|bytes| {
                    // Parse SSE event
                    let text = String::from_utf8_lossy(&bytes);
                    let lines: Vec<&str> = text.lines().collect();
                    
                    for line in lines {
                        if line.starts_with("data: ") {
                            let data = &line[6..]; // Skip "data: "
                            
                            if data == "[DONE]" {
                                continue;
                            }
                            
                            match serde_json::from_str::<ClaudeDeltaResponse>(data) {
                                Ok(response) => return Ok(response),
                                Err(e) => return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
                            }
                        }
                    }
                    
                    Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid SSE event",
                    )) as Box<dyn std::error::Error + Send + Sync>)
                })
        });
        
        Ok(stream)
    }
    
    /// Handle API response
    async fn handle_response(&self, response: Response) -> Result<ClaudeResponse, Box<dyn std::error::Error>> {
        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                let claude_response = response.json::<ClaudeResponse>().await?;
                Ok(claude_response)
            }
            _ => Err(self.handle_error_response(response).await?),
        }
    }
    
    /// Handle error response
    async fn handle_error_response(&self, response: Response) -> Result<Box<dyn std::error::Error>, Box<dyn std::error::Error>> {
        let status = response.status();
        let body = response.text().await?;
        
        // Try to parse as Claude error
        match serde_json::from_str::<ClaudeError>(&body) {
            Ok(error) => {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Claude API error ({}): {}", error.error_type, error.message),
                )))
            }
            Err(_) => {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("HTTP error {}: {}", status, body),
                )))
            }
        }
    }
}

/// Claude API client with convenience methods
#[derive(Clone)]
pub struct ClaudeApiClient {
    /// Inner API client
    api: ClaudeApi,
}

impl ClaudeApiClient {
    /// Create a new Claude API client
    pub fn new(api_key: String, base_url: &str) -> Self {
        Self {
            api: ClaudeApi::new(api_key, base_url),
        }
    }
    
    /// Set API key
    pub fn set_api_key(&mut self, api_key: String) {
        self.api.set_api_key(api_key);
    }
    
    /// Create a new message
    pub async fn create_message(&self, body: &Value) -> Result<ClaudeResponse, Box<dyn std::error::Error>> {
        self.api.create_message(body).await
    }
    
    /// Create a streaming message
    pub async fn create_message_stream(
        &self,
        body: &Value,
    ) -> Result<impl Stream<Item = Result<ClaudeDeltaResponse, Box<dyn std::error::Error + Send + Sync>>>, Box<dyn std::error::Error>> {
        self.api.create_message_stream(body).await
    }
}
