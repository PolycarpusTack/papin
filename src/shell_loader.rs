use lazy_static::lazy_static;
use log::{debug, info, warn};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Manager, Window};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::feature_flags::FeatureFlags;
use crate::utils::config::Config;

/// Represents the different states of the application loading process
#[derive(Debug, Clone, PartialEq)]
pub enum LoadState {
    /// Initial state before loading has started
    NotStarted,
    
    /// Basic shell UI is loading (target: <100ms)
    ShellLoading,
    
    /// Basic shell UI is loaded and visible to user
    ShellReady,
    
    /// Core services are being initialized (auth, settings, etc)
    CoreServicesLoading,
    
    /// Core services are ready
    CoreServicesReady,
    
    /// Secondary features are being loaded (plugins, extensions, etc)
    SecondaryLoading,
    
    /// Application is fully loaded
    FullyLoaded,
    
    /// An error occurred during loading
    Error(String),
}

/// Structure to manage application loading state and process
pub struct ShellLoader {
    /// Current loading state
    state: Arc<Mutex<LoadState>>,
    
    /// Channel to send loading updates
    tx: Option<Sender<LoadState>>,
    
    /// Main application window
    window: Option<Window>,
    
    /// Feature flags to determine what to load
    feature_flags: FeatureFlags,
    
    /// Loading tasks being processed
    loading_tasks: Vec<JoinHandle<()>>,
    
    /// Timestamp when loading started
    start_time: Option<Instant>,
}

impl ShellLoader {
    /// Create a new ShellLoader instance
    pub fn new(feature_flags: FeatureFlags) -> Self {
        ShellLoader {
            state: Arc::new(Mutex::new(LoadState::NotStarted)),
            tx: None,
            window: None,
            feature_flags,
            loading_tasks: Vec::new(),
            start_time: None,
        }
    }
    
    /// Initialize the loader with a window
    pub fn with_window(mut self, window: Window) -> Self {
        self.window = Some(window);
        self
    }
    
    /// Start the loading process
    pub async fn start(&mut self) -> Receiver<LoadState> {
        let (tx, rx) = mpsc::channel(16);
        self.tx = Some(tx);
        self.start_time = Some(Instant::now());
        
        let state_clone = self.state.clone();
        let tx_clone = self.tx.clone().unwrap();
        let window_clone = self.window.clone();
        let feature_flags = self.feature_flags;
        
        // Spawn the loading process in a separate task
        let handle = tokio::spawn(async move {
            Self::loading_process(state_clone, tx_clone, window_clone, feature_flags).await;
        });
        
        self.loading_tasks.push(handle);
        rx
    }
    
    /// The main loading process, runs asynchronously
    async fn loading_process(
        state: Arc<Mutex<LoadState>>,
        tx: Sender<LoadState>,
        window: Option<Window>,
        feature_flags: FeatureFlags,
    ) {
        // Update state to shell loading
        Self::update_state(&state, &tx, LoadState::ShellLoading).await;
        
        // Wait for minimal shell to be ready (simulated here)
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Shell is ready, make window visible
        Self::update_state(&state, &tx, LoadState::ShellReady).await;
        if let Some(window) = &window {
            window.show().unwrap();
        }
        
        // Load core services
        Self::update_state(&state, &tx, LoadState::CoreServicesLoading).await;
        
        // Initialize essential services
        Self::initialize_core_services(&feature_flags).await;
        
        // Core services ready
        Self::update_state(&state, &tx, LoadState::CoreServicesReady).await;
        
        // If lazy loading is enabled, load secondary features in the background
        if feature_flags.contains(FeatureFlags::LAZY_LOAD) {
            tokio::spawn(async move {
                Self::update_state(&state, &tx, LoadState::SecondaryLoading).await;
                
                // Load non-essential components in the background
                Self::load_secondary_features(&feature_flags).await;
                
                // Everything is loaded
                Self::update_state(&state, &tx, LoadState::FullyLoaded).await;
            });
        } else {
            // Load everything synchronously
            Self::update_state(&state, &tx, LoadState::SecondaryLoading).await;
            Self::load_secondary_features(&feature_flags).await;
            Self::update_state(&state, &tx, LoadState::FullyLoaded).await;
        }
    }
    
    /// Initialize core services required for basic functionality
    async fn initialize_core_services(feature_flags: &FeatureFlags) {
        // Simulate initialization of essential services
        let services = [
            ("config", 30),
            ("auth", 40),
            ("api", 50),
        ];
        
        for (service, delay_ms) in services {
            debug!("Initializing core service: {}", service);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
    }
    
    /// Load secondary features that are not needed for initial startup
    async fn load_secondary_features(feature_flags: &FeatureFlags) {
        // Only load features that are enabled in feature flags
        if feature_flags.contains(FeatureFlags::PLUGINS) {
            debug!("Loading plugins");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        if feature_flags.contains(FeatureFlags::ADVANCED_UI) {
            debug!("Loading advanced UI components");
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        
        if feature_flags.contains(FeatureFlags::HISTORY) {
            debug!("Loading conversation history");
            tokio::time::sleep(Duration::from_millis(80)).await;
        }
    }
    
    /// Update the loading state and send notification through the channel
    async fn update_state(
        state: &Arc<Mutex<LoadState>>,
        tx: &Sender<LoadState>,
        new_state: LoadState,
    ) {
        {
            let mut state_guard = state.lock().unwrap();
            *state_guard = new_state.clone();
        }
        
        // Send state update through channel
        if let Err(e) = tx.send(new_state).await {
            warn!("Failed to send loading state update: {}", e);
        }
    }
    
    /// Get the current loading state
    pub fn state(&self) -> LoadState {
        self.state.lock().unwrap().clone()
    }
    
    /// Get elapsed time since loading started
    pub fn elapsed(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }
    
    /// Check if the application is fully loaded
    pub fn is_fully_loaded(&self) -> bool {
        matches!(self.state(), LoadState::FullyLoaded)
    }
}

// Helper function to create and start the shell loader with default options
pub async fn launch_with_fast_shell(window: Window, config: &Config) -> ShellLoader {
    info!("Launching application with fast shell");
    
    // Determine feature flags based on config
    let mut feature_flags = FeatureFlags::default();
    if config.get_bool("experimental_features").unwrap_or(false) {
        feature_flags |= FeatureFlags::EXPERIMENTAL;
    }
    
    if config.get_bool("lazy_loading").unwrap_or(true) {
        feature_flags |= FeatureFlags::LAZY_LOAD;
    }
    
    // Enable other features based on config
    if config.get_bool("plugins_enabled").unwrap_or(true) {
        feature_flags |= FeatureFlags::PLUGINS;
    }
    
    if config.get_bool("history_enabled").unwrap_or(true) {
        feature_flags |= FeatureFlags::HISTORY;
    }
    
    if config.get_bool("advanced_ui").unwrap_or(true) {
        feature_flags |= FeatureFlags::ADVANCED_UI;
    }
    
    // Create and start the shell loader
    let mut loader = ShellLoader::new(feature_flags).with_window(window);
    loader.start().await;
    
    loader
}