use crate::optimization::{OptimizationManager, MemoryLimits, CacheConfig, MemoryStats, CacheStats};
use std::sync::{Arc, Mutex};
use tauri::{command, State};

/// State for the optimization manager
pub struct OptimizationState {
    manager: Arc<Mutex<Option<OptimizationManager>>>,
}

impl OptimizationState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(None)),
        }
    }
    
    pub fn initialize(&self) {
        let manager = OptimizationManager::new();
        manager.start();
        *self.manager.lock().unwrap() = Some(manager);
    }
    
    pub fn get_manager(&self) -> Option<OptimizationManager> {
        self.manager.lock().unwrap().as_ref().cloned()
    }
}

/// Initialize the optimization manager
#[command]
pub fn init_optimizations(state: State<'_, OptimizationState>) -> Result<String, String> {
    state.initialize();
    Ok("Optimization manager initialized".into())
}

/// Get memory statistics
#[command]
pub fn get_memory_stats(state: State<'_, OptimizationState>) -> Result<MemoryStats, String> {
    match state.get_manager() {
        Some(manager) => Ok(manager.memory_manager().get_stats()),
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Get memory limits
#[command]
pub fn get_memory_limits(state: State<'_, OptimizationState>) -> Result<MemoryLimits, String> {
    match state.get_manager() {
        Some(manager) => Ok(manager.memory_manager().get_limits()),
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Update memory limits
#[command]
pub fn update_memory_limits(
    limits: MemoryLimits,
    state: State<'_, OptimizationState>
) -> Result<String, String> {
    match state.get_manager() {
        Some(manager) => {
            manager.memory_manager().update_limits(limits);
            Ok("Memory limits updated".into())
        }
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Force garbage collection
#[command]
pub fn force_gc(
    aggressive: bool,
    state: State<'_, OptimizationState>
) -> Result<String, String> {
    match state.get_manager() {
        Some(manager) => {
            manager.memory_manager().force_gc(aggressive);
            Ok("Garbage collection performed".into())
        }
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Get API cache statistics
#[command]
pub fn get_api_cache_stats(state: State<'_, OptimizationState>) -> Result<CacheStats, String> {
    match state.get_manager() {
        Some(manager) => Ok(manager.api_cache().get_stats()),
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Get API cache configuration
#[command]
pub fn get_api_cache_config(state: State<'_, OptimizationState>) -> Result<CacheConfig, String> {
    match state.get_manager() {
        Some(manager) => Ok(manager.api_cache().get_config()),
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Update API cache configuration
#[command]
pub fn update_api_cache_config(
    config: CacheConfig,
    state: State<'_, OptimizationState>
) -> Result<String, String> {
    match state.get_manager() {
        Some(manager) => {
            manager.api_cache().update_config(config);
            Ok("API cache configuration updated".into())
        }
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Clear API cache
#[command]
pub fn clear_api_cache(state: State<'_, OptimizationState>) -> Result<String, String> {
    match state.get_manager() {
        Some(manager) => {
            manager.api_cache().clear();
            Ok("API cache cleared".into())
        }
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Get resource cache statistics
#[command]
pub fn get_resource_cache_stats(state: State<'_, OptimizationState>) -> Result<CacheStats, String> {
    match state.get_manager() {
        Some(manager) => Ok(manager.resource_cache().get_stats()),
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Get resource cache configuration
#[command]
pub fn get_resource_cache_config(state: State<'_, OptimizationState>) -> Result<CacheConfig, String> {
    match state.get_manager() {
        Some(manager) => Ok(manager.resource_cache().get_config()),
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Update resource cache configuration
#[command]
pub fn update_resource_cache_config(
    config: CacheConfig,
    state: State<'_, OptimizationState>
) -> Result<String, String> {
    match state.get_manager() {
        Some(manager) => {
            manager.resource_cache().update_config(config);
            Ok("Resource cache configuration updated".into())
        }
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Clear resource cache
#[command]
pub fn clear_resource_cache(state: State<'_, OptimizationState>) -> Result<String, String> {
    match state.get_manager() {
        Some(manager) => {
            manager.resource_cache().clear();
            Ok("Resource cache cleared".into())
        }
        None => Err("Optimization manager not initialized".into()),
    }
}

/// Register optimization commands with Tauri
pub fn register_commands(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(OptimizationState::new());
    
    Ok(())
}