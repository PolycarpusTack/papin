use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

// Plugin settings
#[derive(Serialize, Deserialize)]
struct Settings {
    summary_format: String,
    include_action_items: bool,
    include_decisions: bool,
    include_participants: bool,
    auto_summarize: bool,
}

// Meeting summary
#[derive(Serialize, Deserialize)]
struct MeetingSummary {
    title: String,
    summary: String,
    action_items: Vec<String>,
    decisions: Vec<String>,
    participants: Vec<String>,
}

// Plugin entry point
#[wasm_bindgen]
pub fn init() {
    // Register the plugin
    host::register_plugin();
    
    // Register hooks
    host::register_hook("message:pre-process", pre_process_message);
    host::register_hook("conversation:create", conversation_created);
    
    host::log_message("info", "Meeting Summarizer plugin initialized");
}

// Pre-process message hook
fn pre_process_message(context_ptr: i32) -> i32 {
    // Parse context
    let context_str = host::read_memory(context_ptr);
    let context: Value = serde_json::from_str(&context_str).unwrap();
    
    // Get message content
    let message = context["data"]["message"].as_object().unwrap();
    let content = message["content"].as_str().unwrap();
    
    // Check for summarize command
    if content.starts_with("/summarize") {
        // Get settings
        let settings_str = host::get_settings();
        let settings: Settings = serde_json::from_str(&settings_str).unwrap_or_else(|_| Settings {
            summary_format: "detailed".to_string(),
            include_action_items: true,
            include_decisions: true,
            include_participants: true,
            auto_summarize: false,
        });
        
        // Parse command
        let parts: Vec<&str> = content.split_whitespace().collect();
        
        // Get previous messages to summarize
        let conversation_id = context["data"]["conversation_id"].as_str().unwrap();
        let messages = get_conversation_messages(conversation_id);
        
        if messages.is_empty() {
            // No messages to summarize
            let help_message = "No messages found to summarize. Please provide a meeting transcript.";
            
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
        
        // Get format parameter
        let format = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            settings.summary_format.clone()
        };
        
        // Summarize the transcript
        let summary = summarize_transcript(&messages, &format, &settings);
        
        // Create modified message
        let mut new_message = message.clone();
        new_message["content"] = json!(summary);
        
        // Update context
        let mut new_context = context.clone();
        new_context["data"]["message"] = json!(new_message);
        
        // Write result to memory
        let result = serde_json::to_string(&new_context).unwrap();
        host::write_memory(result.as_ptr(), result.len() as i32)
    } else if content.starts_with("/meeting") {
        // Create new meeting transcript
        let meeting_prompt = "Let's start a new meeting. I'll help you take notes and generate a summary afterward.\n\n\
                             When you're done with the meeting, use the `/summarize` command to generate a summary.";
        
        // Create modified message
        let mut new_message = message.clone();
        new_message["content"] = json!(meeting_prompt);
        
        // Update context
        let mut new_context = context.clone();
        new_context["data"]["message"] = json!(new_message);
        
        // Write result to memory
        let result = serde_json::to_string(&new_context).unwrap();
        host::write_memory(result.as_ptr(), result.len() as i32)
    } else {
        // Check for auto-summarize
        let settings_str = host::get_settings();
        let settings: Settings = serde_json::from_str(&settings_str).unwrap_or_else(|_| Settings {
            summary_format: "detailed".to_string(),
            include_action_items: true,
            include_decisions: true,
            include_participants: true,
            auto_summarize: false,
        });
        
        if settings.auto_summarize && is_meeting_transcript(content) {
            // Auto-summarize meeting transcript
            let summary = summarize_transcript(&vec![content.to_string()], &settings.summary_format, &settings);
            
            // Create modified message
            let mut new_message = message.clone();
            new_message["content"] = json!(format!("{}\n\n{}", content, summary));
            
            // Update context
            let mut new_context = context.clone();
            new_context["data"]["message"] = json!(new_message);
            
            // Write result to memory
            let result = serde_json::to_string(&new_context).unwrap();
            return host::write_memory(result.as_ptr(), result.len() as i32);
        }
        
        // Not a known command, return unchanged
        0
    }
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

// Get messages from a conversation
fn get_conversation_messages(conversation_id: &str) -> Vec<String> {
    // In a real plugin, this would call the host function to get conversation messages
    // For now, we return a mock conversation
    
    vec![
        "John: Hi everyone, thanks for joining this meeting. Today we're going to discuss the Q2 roadmap.".to_string(),
        "Sarah: Sounds good. I have some ideas I'd like to share.".to_string(),
        "Michael: I think we should prioritize the new features over the bug fixes.".to_string(),
        "Sarah: I disagree. We have too many critical bugs that need to be fixed.".to_string(),
        "John: I see both points. Let's decide on a balance. How about we allocate 40% to new features and 60% to bug fixes?".to_string(),
        "Sarah: That sounds reasonable to me.".to_string(),
        "Michael: I can live with that. Let's go with 40/60.".to_string(),
        "John: Great! So the decision is 40% new features, 60% bug fixes for Q2.".to_string(),
        "John: Now, about the team expansion. I think we need to hire two more developers.".to_string(),
        "Sarah: Agreed. I'll start the recruitment process next week.".to_string(),
        "Michael: I'll prepare the onboarding materials.".to_string(),
        "John: Perfect. Sarah, please have the job postings ready by Monday. Michael, please have the onboarding materials ready by the end of the month.".to_string(),
        "Sarah: Will do.".to_string(),
        "Michael: Got it.".to_string(),
        "John: Any other topics we need to discuss?".to_string(),
        "Sarah: I think that covers everything for now.".to_string(),
        "John: Great! Thanks everyone for your time. Meeting adjourned.".to_string(),
    ]
}

// Check if a message appears to be a meeting transcript
fn is_meeting_transcript(content: &str) -> bool {
    // Simple heuristic: check if the message contains multiple speakers
    // and looks like a conversation
    
    // Count number of lines that appear to be speaker turns
    let lines: Vec<&str> = content.lines().collect();
    let mut speaker_turns = 0;
    
    for line in lines.iter() {
        if line.contains(":") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 && !parts[0].is_empty() && !parts[0].contains(" ") {
                speaker_turns += 1;
            }
        }
    }
    
    // If we have at least 5 speaker turns, it's likely a meeting transcript
    speaker_turns >= 5
}

// Extract participants from transcript
fn extract_participants(transcript: &[String]) -> Vec<String> {
    // Extract names from the transcript
    let mut participants = Vec::new();
    
    for line in transcript {
        if line.contains(":") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 && !parts[0].is_empty() {
                // Extract name
                let name = parts[0].trim();
                
                // Add if not already in the list
                if !participants.contains(&name.to_string()) {
                    participants.push(name.to_string());
                }
            }
        }
    }
    
    participants
}

// Extract action items from transcript
fn extract_action_items(transcript: &[String]) -> Vec<String> {
    // Look for phrases that indicate action items
    let mut action_items = Vec::new();
    
    let action_indicators = [
        "will",
        "need to",
        "should",
        "have to",
        "going to",
        "action item",
        "please",
        "task",
        "by tomorrow",
        "next week",
        "by monday",
    ];
    
    for line in transcript {
        let lowercase = line.to_lowercase();
        
        // Check if the line contains any action indicators
        if action_indicators.iter().any(|&indicator| lowercase.contains(indicator)) {
            // Extract the action item
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let speaker = parts[0].trim();
                let content = parts[1].trim();
                
                action_items.push(format!("{}: {}", speaker, content));
            } else {
                action_items.push(line.to_string());
            }
        }
    }
    
    // Filter out duplicates
    action_items.dedup();
    
    action_items
}

// Extract decisions from transcript
fn extract_decisions(transcript: &[String]) -> Vec<String> {
    // Look for phrases that indicate decisions
    let mut decisions = Vec::new();
    
    let decision_indicators = [
        "decided",
        "agreed",
        "decision",
        "consensus",
        "agreement",
        "settled on",
        "concluded",
        "resolved",
        "finalized",
    ];
    
    for line in transcript {
        let lowercase = line.to_lowercase();
        
        // Check if the line contains any decision indicators
        if decision_indicators.iter().any(|&indicator| lowercase.contains(indicator)) {
            // Extract the decision
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let speaker = parts[0].trim();
                let content = parts[1].trim();
                
                decisions.push(format!("{}: {}", speaker, content));
            } else {
                decisions.push(line.to_string());
            }
        }
    }
    
    // Filter out duplicates
    decisions.dedup();
    
    decisions
}

// Summarize a meeting transcript
fn summarize_transcript(transcript: &[String], format: &str, settings: &Settings) -> String {
    // Extract information from transcript
    let participants = if settings.include_participants {
        extract_participants(transcript)
    } else {
        Vec::new()
    };
    
    let action_items = if settings.include_action_items {
        extract_action_items(transcript)
    } else {
        Vec::new()
    };
    
    let decisions = if settings.include_decisions {
        extract_decisions(transcript)
    } else {
        Vec::new()
    };
    
    // Request model use permission
    if host::request_permission("models:use") == 0 {
        return format!("Error: Permission denied for using models");
    }
    
    // Create a summary based on the transcript
    // In a real plugin, this would use Claude to generate a summary
    // Here we use a mock function
    let summary = generate_summary(transcript, format);
    
    // Format the summary based on the requested format
    format_summary(
        &summary,
        &participants,
        &action_items,
        &decisions,
        format,
        settings,
    )
}

// Generate a summary from a transcript
fn generate_summary(transcript: &[String], format: &str) -> String {
    // In a real plugin, this would use Claude to generate a summary
    // For now, we generate a mock summary based on the transcript
    
    match format {
        "concise" => {
            "The team discussed the Q2 roadmap and decided on allocating 40% to new features and 60% to bug fixes. \
             They also agreed to hire two more developers, with Sarah handling recruitment and Michael preparing onboarding materials."
                .to_string()
        }
        "bullet" => {
            "• Team discussed Q2 roadmap\n\
             • Decided on 40% new features, 60% bug fixes\n\
             • Agreed to hire two more developers\n\
             • Sarah will handle recruitment\n\
             • Michael will prepare onboarding materials"
                .to_string()
        }
        _ => {
            // Detailed format
            "During this meeting, the team discussed the Q2 roadmap priorities. There was a debate about \
             whether to focus on new features or bug fixes. Michael preferred prioritizing new features while \
             Sarah advocated for addressing critical bugs. John suggested a compromise of 40% new features and \
             60% bug fixes, which everyone agreed to.\n\n\
             The team also discussed expanding the team by hiring two more developers. Sarah will start the \
             recruitment process next week and have job postings ready by Monday. Michael will prepare the \
             onboarding materials by the end of the month."
                .to_string()
        }
    }
}

// Format the summary
fn format_summary(
    summary: &str,
    participants: &[String],
    action_items: &[String],
    decisions: &[String],
    format: &str,
    settings: &Settings,
) -> String {
    let mut result = String::new();
    
    // Add title
    result.push_str("# Meeting Summary\n\n");
    
    // Add summary
    match format {
        "concise" => {
            result.push_str("## Summary\n\n");
            result.push_str(summary);
            result.push_str("\n\n");
        }
        "bullet" => {
            result.push_str("## Summary\n\n");
            result.push_str(summary);
            result.push_str("\n\n");
        }
        _ => {
            // Detailed format
            result.push_str("## Detailed Summary\n\n");
            result.push_str(summary);
            result.push_str("\n\n");
        }
    }
    
    // Add participants if requested
    if settings.include_participants && !participants.is_empty() {
        result.push_str("## Participants\n\n");
        for participant in participants {
            result.push_str(&format!("- {}\n", participant));
        }
        result.push_str("\n");
    }
    
    // Add decisions if requested
    if settings.include_decisions && !decisions.is_empty() {
        result.push_str("## Key Decisions\n\n");
        for decision in decisions {
            result.push_str(&format!("- {}\n", decision));
        }
        result.push_str("\n");
    }
    
    // Add action items if requested
    if settings.include_action_items && !action_items.is_empty() {
        result.push_str("## Action Items\n\n");
        for action in action_items {
            result.push_str(&format!("- {}\n", action));
        }
        result.push_str("\n");
    }
    
    result
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
