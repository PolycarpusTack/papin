use crate::auto_update::{UpdateManager, UpdaterConfig};
use tauri::{AppHandle, command, State, Manager};
use std::sync::{Arc, Mutex};

/// State for the updater
pub struct UpdaterState {
    manager: Arc<Mutex<Option<UpdateManager>>>,
}

impl UpdaterState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(None)),
        }
    }

    pub fn initialize(&self, app: AppHandle) {
        let manager = UpdateManager::new(app);
        *self.manager.lock().unwrap() = Some(manager);
    }

    pub fn get_manager(&self) -> Option<UpdateManager> {
        self.manager.lock().unwrap().clone()
    }
}

/// Initialize the update manager
#[command]
pub async fn init_updater(app_handle: AppHandle, state: State<'_, UpdaterState>) {
    state.initialize(app_handle);
    if let Some(manager) = state.get_manager() {
        manager.start().await;
    }
}

/// Check for updates manually
#[command]
pub async fn check_for_updates(state: State<'_, UpdaterState>) -> Result<String, String> {
    match state.get_manager() {
        Some(manager) => {
            manager.check_for_updates().await;
            Ok("Update check initiated".into())
        }
        None => Err("Update manager not initialized".into()),
    }
}

/// Get the current updater configuration
#[command]
pub fn get_updater_config(state: State<'_, UpdaterState>) -> Result<UpdaterConfig, String> {
    match state.get_manager() {
        Some(manager) => Ok(manager.get_config()),
        None => Err("Update manager not initialized".into()),
    }
}

/// Update the updater configuration
#[command]
pub async fn update_updater_config(
    config: UpdaterConfig, 
    state: State<'_, UpdaterState>
) -> Result<String, String> {
    match state.get_manager() {
        Some(manager) => {
            manager.update_config(config).await;
            Ok("Configuration updated".into())
        }
        None => Err("Update manager not initialized".into()),
    }
}

/// Register updater commands with Tauri
pub fn register_commands(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(UpdaterState::new());
    
    Ok(())
}