use log::{debug, warn};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

/// Type alias for module initialization function
type InitFn = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send>;

/// Status of a lazy-loaded module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus {
    /// Module is not loaded
    NotLoaded,
    
    /// Module is currently loading
    Loading,
    
    /// Module has been loaded successfully
    Loaded,
    
    /// Module failed to load
    Failed,
}

/// Information about a lazy-loaded module
struct ModuleInfo {
    /// Current status of the module
    status: ModuleStatus,
    
    /// Time when the module was loaded
    load_time: Option<Duration>,
    
    /// Error message if loading failed
    error: Option<String>,
    
    /// Initialization function for the module
    init_fn: Option<InitFn>,
    
    /// Channel to notify when loading is complete
    notify: Option<oneshot::Sender<Result<(), String>>>,
}

/// Manager for lazy-loaded modules
pub struct LazyLoader {
    /// Registered modules
    modules: Arc<Mutex<HashMap<String, ModuleInfo>>>,
}

impl LazyLoader {
    /// Create a new lazy loader
    pub fn new() -> Self {
        LazyLoader {
            modules: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Register a module with the lazy loader
    pub fn register<F, Fut>(&self, name: &str, init_fn: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        let boxed_init = Box::new(move || Box::pin(init_fn()) as Pin<Box<dyn Future<Output = Result<(), String>> + Send>>);
        
        let mut modules = self.modules.lock().unwrap();
        modules.insert(name.to_string(), ModuleInfo {
            status: ModuleStatus::NotLoaded,
            load_time: None,
            error: None,
            init_fn: Some(boxed_init),
            notify: None,
        });
        
        debug!("Registered lazy-loaded module: {}", name);
    }
    
    /// Start loading a module
    pub async fn load(&self, name: &str) -> Result<(), String> {
        let (init_fn, tx) = {
            let mut modules = self.modules.lock().unwrap();
            
            let module = modules.get_mut(name).ok_or_else(|| {
                format!("Module not registered: {}", name)
            })?;
            
            if module.status == ModuleStatus::Loaded {
                return Ok(());
            }
            
            if module.status == ModuleStatus::Loading {
                let (tx, rx) = oneshot::channel();
                module.notify = Some(tx);
                return rx.await.unwrap_or_else(|_| {
                    Err(format!("Module loading was canceled: {}", name))
                });
            }
            
            let init_fn = module.init_fn.take().ok_or_else(|| {
                format!("Module initialization function not found: {}", name)
            })?;
            
            module.status = ModuleStatus::Loading;
            
            let (tx, rx) = oneshot::channel();
            module.notify = Some(tx);
            
            (init_fn, rx)
        };
        
        // Execute initialization function in a separate task
        let modules_clone = self.modules.clone();
        let name_clone = name.to_string();
        
        tokio::spawn(async move {
            let start_time = Instant::now();
            let result = init_fn().await;
            let elapsed = start_time.elapsed();
            
            // Update module status
            let mut modules = modules_clone.lock().unwrap();
            if let Some(module) = modules.get_mut(&name_clone) {
                module.load_time = Some(elapsed);
                
                if let Err(ref e) = result {
                    module.status = ModuleStatus::Failed;
                    module.error = Some(e.clone());
                    warn!("Failed to load module {}: {}", name_clone, e);
                } else {
                    module.status = ModuleStatus::Loaded;
                    debug!("Loaded module {} in {}ms", name_clone, elapsed.as_millis());
                }
                
                // Notify waiting tasks
                if let Some(tx) = module.notify.take() {
                    let _ = tx.send(result.clone());
                }
            }
        });
        
        // Wait for the module to be loaded
        tx.await.unwrap_or_else(|_| {
            Err(format!("Module loading was canceled: {}", name))
        })
    }
    
    /// Get the status of a module
    pub fn status(&self, name: &str) -> ModuleStatus {
        let modules = self.modules.lock().unwrap();
        modules.get(name).map(|m| m.status).unwrap_or(ModuleStatus::NotLoaded)
    }
    
    /// Check if a module is loaded
    pub fn is_loaded(&self, name: &str) -> bool {
        self.status(name) == ModuleStatus::Loaded
    }
    
    /// Get the load time of a module
    pub fn load_time(&self, name: &str) -> Option<Duration> {
        let modules = self.modules.lock().unwrap();
        modules.get(name).and_then(|m| m.load_time)
    }
    
    /// Get the error message for a failed module
    pub fn error(&self, name: &str) -> Option<String> {
        let modules = self.modules.lock().unwrap();
        modules.get(name).and_then(|m| m.error.clone())
    }
    
    /// Get all registered modules and their status
    pub fn get_all_modules(&self) -> HashMap<String, ModuleStatus> {
        let modules = self.modules.lock().unwrap();
        modules.iter().map(|(name, info)| (name.clone(), info.status)).collect()
    }
}

// Global lazy loader instance
lazy_static::lazy_static! {
    static ref GLOBAL_LAZY_LOADER: LazyLoader = LazyLoader::new();
}

/// Get the global lazy loader instance
pub fn get_lazy_loader() -> &'static LazyLoader {
    &GLOBAL_LAZY_LOADER
}

/// Register a module with the global lazy loader
pub fn register_module<F, Fut>(name: &str, init_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), String>> + Send + 'static,
{
    GLOBAL_LAZY_LOADER.register(name, init_fn);
}

/// Load a module from the global lazy loader
pub async fn load_module(name: &str) -> Result<(), String> {
    GLOBAL_LAZY_LOADER.load(name).await
}

/// Check if a module is loaded in the global lazy loader
pub fn is_module_loaded(name: &str) -> bool {
    GLOBAL_LAZY_LOADER.is_loaded(name)
}