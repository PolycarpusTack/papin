use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name
    pub name: String,
    
    /// Tool description
    pub description: String,
    
    /// Schema in JSON Schema format
    pub schema: serde_json::Value,
}

/// Tool call
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    
    /// Tool name
    pub name: String,
    
    /// Tool arguments
    pub arguments: serde_json::Value,
}

/// Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool call ID
    pub tool_call_id: String,
    
    /// Tool name
    pub name: String,
    
    /// Tool result
    pub result: serde_json::Value,
}

impl Tool {
    /// Create a new tool definition
    pub fn new(name: impl Into<String>, description: impl Into<String>, schema: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            schema,
        }
    }
    
    /// Create a simple function tool with string input and output
    pub fn simple_function(name: impl Into<String>, description: impl Into<String>) -> Self {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "The input to the function"
                }
            },
            "required": ["input"]
        });
        
        Self::new(name, description, schema)
    }
    
    /// Get common tools
    pub fn common_tools() -> Vec<Self> {
        vec![
            Self::simple_function(
                "web_search",
                "Search the web for information"
            ),
            Self::simple_function(
                "calculator",
                "Perform mathematical calculations"
            ),
            Self::simple_function(
                "file_read",
                "Read the contents of a file"
            ),
            Self::simple_function(
                "file_write",
                "Write content to a file"
            ),
        ]
    }
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments,
        }
    }
}

impl ToolResult {
    /// Create a new tool result
    pub fn new(tool_call_id: impl Into<String>, name: impl Into<String>, result: serde_json::Value) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            name: name.into(),
            result,
        }
    }
    
    /// Create a success result
    pub fn success(tool_call_id: impl Into<String>, name: impl Into<String>, data: impl Into<String>) -> Self {
        Self::new(
            tool_call_id,
            name,
            serde_json::json!({
                "status": "success",
                "data": data.into()
            })
        )
    }
    
    /// Create an error result
    pub fn error(tool_call_id: impl Into<String>, name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            tool_call_id,
            name,
            serde_json::json!({
                "status": "error",
                "message": message.into()
            })
        )
    }
}
