use std::fmt;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

use crate::plugins::types::HookContext;

/// Hook registry
pub struct HookRegistry {
    /// Registered hooks by type
    hooks: RwLock<HashMap<HookType, Vec<HookRegistration>>>,
}

/// Hook registration
#[derive(Debug, Clone)]
pub struct HookRegistration {
    /// Plugin ID
    pub plugin_id: String,
    /// Instance ID
    pub instance_id: String,
    /// Priority (lower numbers run first)
    pub priority: i32,
}

/// Hook types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    /// Process a message before sending to Claude
    MessagePreProcess,
    /// Process a message after receiving from Claude
    MessagePostProcess,
    /// Called when a new conversation is created
    ConversationCreate,
    /// Called when a conversation is opened
    ConversationOpen,
    /// Called when a conversation is closed
    ConversationClose,
    /// Called when the application starts
    ApplicationStart,
    /// Called when the application shuts down
    ApplicationShutdown,
    /// Custom UI rendering
    UiRender,
}

impl fmt::Display for HookType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HookType::MessagePreProcess => write!(f, "message:pre-process"),
            HookType::MessagePostProcess => write!(f, "message:post-process"),
            HookType::ConversationCreate => write!(f, "conversation:create"),
            HookType::ConversationOpen => write!(f, "conversation:open"),
            HookType::ConversationClose => write!(f, "conversation:close"),
            HookType::ApplicationStart => write!(f, "application:start"),
            HookType::ApplicationShutdown => write!(f, "application:shutdown"),
            HookType::UiRender => write!(f, "ui:render"),
        }
    }
}

impl HookType {
    /// Parse a hook type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "message:pre-process" => Some(HookType::MessagePreProcess),
            "message:post-process" => Some(HookType::MessagePostProcess),
            "conversation:create" => Some(HookType::ConversationCreate),
            "conversation:open" => Some(HookType::ConversationOpen),
            "conversation:close" => Some(HookType::ConversationClose),
            "application:start" => Some(HookType::ApplicationStart),
            "application:shutdown" => Some(HookType::ApplicationShutdown),
            "ui:render" => Some(HookType::UiRender),
            _ => None,
        }
    }
}

impl HookRegistry {
    /// Create a new hook registry
    pub fn new() -> Self {
        Self {
            hooks: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a hook
    pub async fn register_hook(&self, hook_type: HookType, plugin_id: &str, 
                              instance_id: &str, priority: i32) -> Result<(), String> {
        let mut hooks = self.hooks.write().await;
        
        // Get or create hook list
        let hook_list = hooks.entry(hook_type).or_insert_with(Vec::new);
        
        // Check if already registered
        for hook in hook_list.iter() {
            if hook.plugin_id == plugin_id && hook.instance_id == instance_id {
                return Err(format!("Hook {:?} already registered for plugin {}", hook_type, plugin_id));
            }
        }
        
        // Register hook
        hook_list.push(HookRegistration {
            plugin_id: plugin_id.to_string(),
            instance_id: instance_id.to_string(),
            priority,
        });
        
        // Sort by priority
        hook_list.sort_by_key(|h| h.priority);
        
        log::info!("Registered hook {:?} for plugin {} (instance {})", hook_type, plugin_id, instance_id);
        Ok(())
    }
    
    /// Unregister a hook
    pub async fn unregister_hook(&self, hook_type: HookType, plugin_id: &str, 
                                instance_id: &str) -> Result<(), String> {
        let mut hooks = self.hooks.write().await;
        
        // Get hook list
        if let Some(hook_list) = hooks.get_mut(&hook_type) {
            // Find hook
            let pos = hook_list.iter().position(|h| {
                h.plugin_id == plugin_id && h.instance_id == instance_id
            });
            
            // Remove if found
            if let Some(idx) = pos {
                hook_list.remove(idx);
                log::info!("Unregistered hook {:?} for plugin {} (instance {})", hook_type, plugin_id, instance_id);
                return Ok(());
            }
        }
        
        Err(format!("Hook {:?} not registered for plugin {}", hook_type, plugin_id))
    }
    
    /// Unregister all hooks for a plugin instance
    pub async fn unregister_all_hooks(&self, instance_id: &str) -> Result<(), String> {
        let mut hooks = self.hooks.write().await;
        
        // Process all hook types
        for (hook_type, hook_list) in hooks.iter_mut() {
            // Remove all hooks for this instance
            hook_list.retain(|h| h.instance_id != instance_id);
        }
        
        log::info!("Unregistered all hooks for plugin instance {}", instance_id);
        Ok(())
    }
    
    /// Get hooks for a specific type
    pub async fn get_hooks(&self, hook_type: HookType) -> Vec<HookRegistration> {
        let hooks = self.hooks.read().await;
        
        if let Some(hook_list) = hooks.get(&hook_type) {
            hook_list.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Execute hooks of a specific type
    pub async fn execute_hooks(&self, hook_type: HookType, 
                              context: &mut HookContext) -> Result<(), String> {
        log::debug!("Executing hooks for {:?}", hook_type);
        
        // Get plugin manager
        let plugin_manager = crate::plugins::get_plugin_manager();
        let plugin_manager = plugin_manager.read().await;
        
        // Skip if plugins are disabled
        if !plugin_manager.is_enabled() {
            return Ok(());
        }
        
        // Get sandbox manager (assumed to be available through plugin manager)
        // Note: In a real implementation, this would use the actual sandbox manager
        
        // Get hooks
        let hooks = self.get_hooks(hook_type).await;
        
        // Execute each hook
        for hook in hooks {
            // Create hook context for this plugin
            let mut plugin_context = HookContext {
                plugin_id: hook.plugin_id.clone(),
                hook_name: hook_type.to_string(),
                data: context.data.clone(),
            };
            
            // Execute hook through sandbox
            // In a real implementation, this would call the sandbox manager
            log::debug!("Executing hook {:?} for plugin {} (instance {})", 
                       hook_type, hook.plugin_id, hook.instance_id);
                       
            // Update context with results
            // In a real implementation, this would update the context with the results from the hook
        }
        
        Ok(())
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}
