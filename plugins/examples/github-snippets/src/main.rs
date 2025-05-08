use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::collections::HashMap;

// Plugin settings
#[derive(Serialize, Deserialize)]
struct Settings {
    github_token: String,
    default_branch: String,
    code_block_style: String,
    max_line_count: u32,
    include_metadata: bool,
}

// GitHub file content response
#[derive(Serialize, Deserialize)]
struct GitHubFileContent {
    content: String,
    encoding: String,
    url: String,
    sha: String,
    size: u32,
}

// GitHub error response
#[derive(Serialize, Deserialize)]
struct GitHubError {
    message: String,
    documentation_url: Option<String>,
}

// Plugin entry point
#[wasm_bindgen]
pub fn init() {
    // Register the plugin
    host::register_plugin();
    
    // Register hooks
    host::register_hook("message:pre-process", pre_process_message);
    host::register_hook("message:post-process", post_process_message);
    
    host::log_message("info", "GitHub Code Snippets plugin initialized");
}

// Pre-process message hook
fn pre_process_message(context_ptr: i32) -> i32 {
    // Parse context
    let context_str = host::read_memory(context_ptr);
    let context: Value = serde_json::from_str(&context_str).unwrap();
    
    // Get message content
    let message = context["data"]["message"].as_object().unwrap();
    let content = message["content"].as_str().unwrap();
    
    // Check for GitHub command
    if content.starts_with("/github") {
        // Get settings
        let settings_str = host::get_settings();
        let settings: Settings = serde_json::from_str(&settings_str).unwrap_or_else(|_| Settings {
            github_token: String::new(),
            default_branch: "main".to_string(),
            code_block_style: "github".to_string(),
            max_line_count: 100,
            include_metadata: true,
        });
        
        // Parse command
        let parts: Vec<&str> = content.split_whitespace().collect();
        
        // Check command format
        if parts.len() < 2 {
            let help_message = "Usage: /github [owner]/[repo]/[path/to/file] [branch/tag] [start_line-end_line]\n\
                               Example: /github microsoft/vscode/src/vs/editor/editor.main.ts main 10-20";
            
            // Create modified message
            let mut new_message = message.clone();
            new_message["content"] = json!(help_message);
            
            // Update context
            let mut new_context = context.clone();
            new_context["data"]["message"] = json!(new_message);
            
            // Write result to memory
            let result = serde_json::to_string(&new_context).unwrap();
            return host::write_memory(result.as_ptr(), result.len() as i32);
        }
        
        // Parse repository path
        let repo_path = parts[1];
        let repo_parts: Vec<&str> = repo_path.split('/').collect();
        
        // Validate repository path
        if repo_parts.len() < 3 {
            let error_message = format!("Invalid repository path: {}. Format should be [owner]/[repo]/[path/to/file]", repo_path);
            
            // Create modified message
            let mut new_message = message.clone();
            new_message["content"] = json!(error_message);
            
            // Update context
            let mut new_context = context.clone();
            new_context["data"]["message"] = json!(new_message);
            
            // Write result to memory
            let result = serde_json::to_string(&new_context).unwrap();
            return host::write_memory(result.as_ptr(), result.len() as i32);
        }
        
        // Extract owner and repo
        let owner = repo_parts[0];
        let repo = repo_parts[1];
        
        // Extract file path
        let file_path = repo_parts[2..].join("/");
        
        // Get branch (optional)
        let branch = if parts.len() > 2 {
            parts[2].to_string()
        } else {
            settings.default_branch.clone()
        };
        
        // Get line range (optional)
        let line_range = if parts.len() > 3 {
            parse_line_range(parts[3], settings.max_line_count)
        } else {
            (0, settings.max_line_count)
        };
        
        // Fetch code from GitHub
        let code_snippet = fetch_github_code(owner, repo, &file_path, &branch, line_range, &settings);
        
        // Create modified message
        let mut new_message = message.clone();
        new_message["content"] = json!(code_snippet);
        
        // Update context
        let mut new_context = context.clone();
        new_context["data"]["message"] = json!(new_message);
        
        // Write result to memory
        let result = serde_json::to_string(&new_context).unwrap();
        host::write_memory(result.as_ptr(), result.len() as i32)
    } else {
        // Not a GitHub command, return unchanged
        0
    }
}

// Post-process message hook
fn post_process_message(context_ptr: i32) -> i32 {
    // This hook doesn't modify post-processed messages
    0
}

// Parse line range
fn parse_line_range(range_str: &str, max_lines: u32) -> (u32, u32) {
    let parts: Vec<&str> = range_str.split('-').collect();
    
    if parts.len() == 2 {
        let start = parts[0].parse::<u32>().unwrap_or(1);
        let end = parts[1].parse::<u32>().unwrap_or(start + max_lines);
        
        // Ensure end is not too far from start
        let end = std::cmp::min(end, start + max_lines);
        
        (start, end)
    } else if parts.len() == 1 {
        let line = parts[0].parse::<u32>().unwrap_or(1);
        (line, line)
    } else {
        (1, max_lines)
    }
}

// Fetch code from GitHub
fn fetch_github_code(owner: &str, repo: &str, path: &str, branch: &str, line_range: (u32, u32), settings: &Settings) -> String {
    // Request permission for GitHub API
    if host::request_permission("network:api.github.com") == 0 {
        return format!("Error: Permission denied for accessing api.github.com");
    }
    
    // Create GitHub API URL
    let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, path);
    
    // Create headers
    let mut headers = HashMap::new();
    headers.insert("Accept".to_string(), "application/vnd.github.v3.raw".to_string());
    
    // Add authentication if token is provided
    if !settings.github_token.is_empty() {
        headers.insert("Authorization".to_string(), format!("token {}", settings.github_token));
    }
    
    // Convert headers to JSON
    let headers_json = serde_json::to_string(&headers).unwrap();
    
    // Add branch as query parameter
    let url_with_branch = format!("{}?ref={}", url, branch);
    
    // Make request to GitHub API
    // In a real plugin, this would be a real HTTP request
    // Here we mock the GitHub API
    let response = mock_github_api(&url_with_branch, &headers_json);
    
    // Check if request was successful
    if response.contains("error") {
        // Parse error
        let error: GitHubError = serde_json::from_str(&response).unwrap();
        
        return format!("GitHub API Error: {}", error.message);
    }
    
    // Extract file content
    let content = response;
    
    // Extract lines
    let lines: Vec<&str> = content.lines().collect();
    
    // Apply line range
    let start_line = line_range.0 as usize;
    let end_line = std::cmp::min(line_range.1 as usize, lines.len());
    
    let selected_lines = if start_line <= end_line && start_line <= lines.len() {
        let start_idx = start_line.saturating_sub(1); // Convert to 0-based index
        let end_idx = std::cmp::min(end_line, lines.len());
        
        lines[start_idx..end_idx].join("\n")
    } else {
        content.clone()
    };
    
    // Format code block based on style
    format_code_block(&selected_lines, path, owner, repo, branch, line_range, settings)
}

// Format code block
fn format_code_block(code: &str, path: &str, owner: &str, repo: &str, branch: &str, line_range: (u32, u32), settings: &Settings) -> String {
    // Detect language from file extension
    let language = detect_language(path);
    
    match settings.code_block_style.as_str() {
        "github" => {
            // GitHub style
            let mut result = String::new();
            
            if settings.include_metadata {
                result.push_str(&format!("```{} File: {}/{}/{}/blob/{}/{} (lines {}-{})\n", 
                                      language, owner, repo, path, branch, line_range.0, line_range.1));
            } else {
                result.push_str(&format!("```{}\n", language));
            }
            
            result.push_str(code);
            result.push_str("\n```");
            
            result
        }
        "detailed" => {
            // Detailed style
            let mut result = String::new();
            
            result.push_str(&format!("# Code from GitHub\n\n"));
            
            if settings.include_metadata {
                result.push_str(&format!("**Repository:** {}/{}\n", owner, repo));
                result.push_str(&format!("**File:** {}\n", path));
                result.push_str(&format!("**Branch:** {}\n", branch));
                result.push_str(&format!("**Lines:** {}-{}\n\n", line_range.0, line_range.1));
            }
            
            result.push_str(&format!("```{}\n", language));
            result.push_str(code);
            result.push_str("\n```");
            
            result
        }
        _ => {
            // Simple style
            let mut result = String::new();
            
            result.push_str(&format!("```{}\n", language));
            result.push_str(code);
            result.push_str("\n```");
            
            result
        }
    }
}

// Detect language from file extension
fn detect_language(path: &str) -> String {
    let parts: Vec<&str> = path.split('.').collect();
    
    if parts.len() < 2 {
        return "".to_string();
    }
    
    let extension = parts.last().unwrap().to_lowercase();
    
    match extension.as_str() {
        "rs" => "rust",
        "js" => "javascript",
        "ts" => "typescript",
        "py" => "python",
        "java" => "java",
        "c" => "c",
        "cpp" => "cpp",
        "h" => "c",
        "hpp" => "cpp",
        "cs" => "csharp",
        "go" => "go",
        "rb" => "ruby",
        "php" => "php",
        "html" => "html",
        "css" => "css",
        "md" => "markdown",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "sql" => "sql",
        "sh" => "bash",
        "bat" => "batch",
        "ps1" => "powershell",
        _ => "",
    }.to_string()
}

// Mock GitHub API (for demonstration)
fn mock_github_api(url: &str, headers: &str) -> String {
    // In a real plugin, this would make an actual HTTP request
    // For now, we return a mock response
    
    // Extract owner, repo, and path from URL
    let url_parts: Vec<&str> = url.split('/').collect();
    
    if url_parts.len() < 7 {
        return json!({
            "error": true,
            "message": "Invalid URL format",
            "documentation_url": "https://docs.github.com/en/rest"
        }).to_string();
    }
    
    // Extract path
    let path = url_parts[7..].join("/");
    
    // Remove query parameters
    let path = path.split('?').next().unwrap();
    
    // Generate mock content based on path
    let extension = path.split('.').last().unwrap_or("");
    
    match extension {
        "rs" => "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
        "js" => "function hello() {\n    console.log(\"Hello, world!\");\n}".to_string(),
        "py" => "def main():\n    print(\"Hello, world!\")\n\nif __name__ == \"__main__\":\n    main()".to_string(),
        _ => "// Sample content for demonstration\n// This would be the actual file content from GitHub".to_string(),
    }
}

// Host function imports
#[wasm_bindgen]
extern "C" {
    pub mod host {
        pub fn register_plugin() -> i32;
        pub fn register_hook(hook_name: &str, callback_ptr: fn(i32) -> i32) -> i32;
        pub fn log_message(level: &str, message: &str) -> i32;
        pub fn read_memory(ptr: i32) -> String;
        pub fn write_memory(ptr: i32, len: i32) -> i32;
        pub fn get_settings() -> String;
        pub fn request_permission(permission: &str) -> i32;
        pub fn http_request(url: &str, method: &str, headers: &str, body: &str) -> i32;
    }
}
