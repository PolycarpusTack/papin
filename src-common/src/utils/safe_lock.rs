use std::fmt::{Debug, Display};
use std::sync::{Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use log::{error, warn};

/// A trait for safely handling lock operations on Mutex and RwLock types
pub trait SafeLock<T> {
    /// Safely lock a mutex or rwlock, logging errors instead of panicking
    fn safe_lock(&self) -> Result<T, String>;
    
    /// Lock with a custom error message
    fn safe_lock_with_context(&self, context: &str) -> Result<T, String>;
}

impl<T> SafeLock<MutexGuard<'_, T>> for Mutex<T> {
    fn safe_lock(&self) -> Result<MutexGuard<'_, T>, String> {
        self.lock().map_err(|e: PoisonError<MutexGuard<'_, T>>| {
            let err_msg = format!("Failed to acquire mutex lock: {}", e);
            error!("{}", err_msg);
            err_msg
        })
    }
    
    fn safe_lock_with_context(&self, context: &str) -> Result<MutexGuard<'_, T>, String> {
        self.lock().map_err(|e: PoisonError<MutexGuard<'_, T>>| {
            let err_msg = format!("Failed to acquire mutex lock ({}): {}", context, e);
            error!("{}", err_msg);
            err_msg
        })
    }
}

impl<T> SafeLock<RwLockReadGuard<'_, T>> for RwLock<T> {
    fn safe_lock(&self) -> Result<RwLockReadGuard<'_, T>, String> {
        self.read().map_err(|e: PoisonError<RwLockReadGuard<'_, T>>| {
            let err_msg = format!("Failed to acquire read lock: {}", e);
            error!("{}", err_msg);
            err_msg
        })
    }
    
    fn safe_lock_with_context(&self, context: &str) -> Result<RwLockReadGuard<'_, T>, String> {
        self.read().map_err(|e: PoisonError<RwLockReadGuard<'_, T>>| {
            let err_msg = format!("Failed to acquire read lock ({}): {}", context, e);
            error!("{}", err_msg);
            err_msg
        })
    }
}

impl<T> SafeLock<RwLockWriteGuard<'_, T>> for RwLock<T> {
    fn safe_lock(&self) -> Result<RwLockWriteGuard<'_, T>, String> {
        self.write().map_err(|e: PoisonError<RwLockWriteGuard<'_, T>>| {
            let err_msg = format!("Failed to acquire write lock: {}", e);
            error!("{}", err_msg);
            err_msg
        })
    }
    
    fn safe_lock_with_context(&self, context: &str) -> Result<RwLockWriteGuard<'_, T>, String> {
        self.write().map_err(|e: PoisonError<RwLockWriteGuard<'_, T>>| {
            let err_msg = format!("Failed to acquire write lock ({}): {}", context, e);
            error!("{}", err_msg);
            err_msg
        })
    }
}