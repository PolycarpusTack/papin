use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

// Plugin settings
#[derive(Serialize, Deserialize)]
struct Settings {
    api_key: String,
    source_language: String,
    target_language: String,
    auto_translate: bool,
    show_original: bool,
}

// Translation request
#[derive(Serialize, Deserialize)]
struct TranslationRequest {
    text: String,
    source_language: String,
    target_language: String,
    api_key: String,
}

// Translation response
#[derive(Serialize, Deserialize)]
struct TranslationResponse {
    translated_text: String,
    detected_language: Option<String>,
    confidence: Option<f32>,
}

// Message hook context
#[derive(Serialize, Deserialize)]
struct MessageContext {
    conversation_id: String,
    message_id: String,
    role: String,
    content: String,
}

// Plugin entry point
#[wasm_bindgen]
pub fn init() {
    // Register the plugin
    host::register_plugin();
    
    // Register hooks
    host::register_hook("message:pre-process", pre_process_message);
    host::register_hook("message:post-process", post_process_message);
    host::register_hook("conversation:create", conversation_created);
    
    host::log_message("info", "Translation plugin initialized");
}

// Pre-process message hook
fn pre_process_message(context_ptr: i32) -> i32 {
    // Parse context
    let context_str = host::read_memory(context_ptr);
    let context: Value = serde_json::from_str(&context_str).unwrap();
    
    // Get settings
    let settings_str = host::get_settings();
    let settings: Settings = serde_json::from_str(&settings_str).unwrap_or_else(|_| Settings {
        api_key: String::new(),
        source_language: "auto".to_string(),
        target_language: "en".to_string(),
        auto_translate: false,
        show_original: true,
    });
    
    // Check if auto-translate is enabled
    if !settings.auto_translate {
        // Auto-translate disabled, return unchanged
        return 0;
    }
    
    // Get message content
    let message = context["data"]["message"].as_object().unwrap();
    let content = message["content"].as_str().unwrap();
    
    // Check for translation command
    if content.starts_with("/translate") {
        // Parse command
        let parts: Vec<&str> = content.split_whitespace().collect();
        
        // Get target language
        let target_language = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            settings.target_language.clone()
        };
        
        // Get text to translate
        let text = if parts.len() > 2 {
            parts[2..].join(" ")
        } else {
            // No text provided, return unchanged
            return 0;
        };
        
        // Translate the text
        let translated = translate(&text, &settings.source_language, &target_language, &settings.api_key);
        
        // Create modified message
        let mut new_message = message.clone();
        if settings.show_original {
            new_message["content"] = json!(format!("Translation ({} -> {}):\n\nOriginal: {}\nTranslated: {}", 
                                                settings.source_language, target_language, text, translated));
        } else {
            new_message["content"] = json!(translated);
        }
        
        // Update context
        let mut new_context = context.clone();
        new_context["data"]["message"] = json!(new_message);
        
        // Write result to memory
        let result = serde_json::to_string(&new_context).unwrap();
        host::write_memory(result.as_ptr(), result.len() as i32)
    } else {
        // Not a translation command, return unchanged
        0
    }
}

// Post-process message hook
fn post_process_message(context_ptr: i32) -> i32 {
    // Parse context
    let context_str = host::read_memory(context_ptr);
    let context: Value = serde_json::from_str(&context_str).unwrap();
    
    // Get settings
    let settings_str = host::get_settings();
    let settings: Settings = serde_json::from_str(&settings_str).unwrap_or_else(|_| Settings {
        api_key: String::new(),
        source_language: "auto".to_string(),
        target_language: "en".to_string(),
        auto_translate: false,
        show_original: true,
    });
    
    // Check if auto-translate is enabled
    if !settings.auto_translate {
        // Auto-translate disabled, return unchanged
        return 0;
    }
    
    // Get message content
    let message = context["data"]["message"].as_object().unwrap();
    let role = message["role"].as_str().unwrap();
    
    // Only translate assistant messages
    if role != "assistant" {
        return 0;
    }
    
    let content = message["content"].as_str().unwrap();
    
    // Translate the text
    let translated = translate(content, &settings.source_language, &settings.target_language, &settings.api_key);
    
    // Create modified message
    let mut new_message = message.clone();
    if settings.show_original {
        new_message["content"] = json!(format!("Original: {}\n\nTranslation ({} -> {}):\n{}", 
                                            content, settings.source_language, settings.target_language, translated));
    } else {
        new_message["content"] = json!(translated);
    }
    
    // Update context
    let mut new_context = context.clone();
    new_context["data"]["message"] = json!(new_message);
    
    // Write result to memory
    let result = serde_json::to_string(&new_context).unwrap();
    host::write_memory(result.as_ptr(), result.len() as i32)
}

// Conversation created hook
fn conversation_created(context_ptr: i32) -> i32 {
    // Parse context
    let context_str = host::read_memory(context_ptr);
    let context: Value = serde_json::from_str(&context_str).unwrap();
    
    // Get conversation ID
    let conversation_id = context["data"]["conversation"]["id"].as_str().unwrap();
    
    host::log_message("info", &format!("New conversation created: {}", conversation_id));
    
    // No changes needed
    0
}

// Translate text
fn translate(text: &str, source_language: &str, target_language: &str, api_key: &str) -> String {
    // Check if API key is provided
    if api_key.is_empty() {
        host::log_message("error", "Translation API key not provided");
        return format!("[Translation failed: API key not provided] {}", text);
    }
    
    // Create translation request
    let request = TranslationRequest {
        text: text.to_string(),
        source_language: source_language.to_string(),
        target_language: target_language.to_string(),
        api_key: api_key.to_string(),
    };
    
    // Convert to JSON
    let request_json = serde_json::to_string(&request).unwrap();
    
    // Send request to translation API
    // In a real plugin, this would make an actual HTTP request
    // Here we mock the translation service
    let response = mock_translation_service(&request);
    
    // Return translated text
    response.translated_text
}

// Mock translation service (for demonstration)
fn mock_translation_service(request: &TranslationRequest) -> TranslationResponse {
    // In a real plugin, this would call the actual translation API
    // For now, we just append a prefix indicating the translation
    TranslationResponse {
        translated_text: format!("[{} translation] {}", request.target_language, request.text),
        detected_language: Some("en".to_string()),
        confidence: Some(0.95),
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
