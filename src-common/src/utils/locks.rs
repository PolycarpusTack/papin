use std::sync::{Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::fmt::{Display, Debug};
use thiserror::Error;

/// Common error type for lock-related operations
#[derive(Error, Debug)]
pub enum LockError {
    #[error("Failed to acquire lock: {0}")]
    AcquireFailed(String),
    
    #[error("Lock was poisoned: {0}")]
    Poisoned(String),
    
    #[error("Operation failed while holding lock: {0}")]
    OperationFailed(String),
}

/// Safe extension trait for Mutex with better error handling
pub trait SafeLock<T> {
    /// Safely lock a mutex, returning a Result rather than using unwrap
    fn safe_lock(&self) -> Result<MutexGuard<'_, T>, LockError>;
}

impl<T> SafeLock<T> for Mutex<T> 
where
    T: Debug
{
    fn safe_lock(&self) -> Result<MutexGuard<'_, T>, LockError> {
        self.lock().map_err(|e| {
            LockError::Poisoned(format!("Mutex poisoned (previous thread panicked): {:?}", e))
        })
    }
}

/// Handle PoisonError with a more descriptive message
impl<T> From<PoisonError<MutexGuard<'_, T>>> for LockError {
    fn from(err: PoisonError<MutexGuard<'_, T>>) -> Self {
        LockError::Poisoned(format!("Mutex poisoned: {:?}", err))
    }
}

/// Safe extension trait for RwLock with better error handling
pub trait SafeRwLock<T> {
    /// Safely acquire a read lock, returning a Result rather than using unwrap
    fn safe_read(&self) -> Result<RwLockReadGuard<'_, T>, LockError>;
    
    /// Safely acquire a write lock, returning a Result rather than using unwrap
    fn safe_write(&self) -> Result<RwLockWriteGuard<'_, T>, LockError>;
}

impl<T> SafeRwLock<T> for RwLock<T>
where
    T: Debug
{
    fn safe_read(&self) -> Result<RwLockReadGuard<'_, T>, LockError> {
        self.read().map_err(|e| {
            LockError::Poisoned(format!("RwLock poisoned during read (previous thread panicked): {:?}", e))
        })
    }
    
    fn safe_write(&self) -> Result<RwLockWriteGuard<'_, T>, LockError> {
        self.write().map_err(|e| {
            LockError::Poisoned(format!("RwLock poisoned during write (previous thread panicked): {:?}", e))
        })
    }
}

/// Implementation of From for RwLock poison errors
impl<T> From<PoisonError<RwLockReadGuard<'_, T>>> for LockError {
    fn from(err: PoisonError<RwLockReadGuard<'_, T>>) -> Self {
        LockError::Poisoned(format!("RwLock read lock poisoned: {:?}", err))
    }
}

impl<T> From<PoisonError<RwLockWriteGuard<'_, T>>> for LockError {
    fn from(err: PoisonError<RwLockWriteGuard<'_, T>>) -> Self {
        LockError::Poisoned(format!("RwLock write lock poisoned: {:?}", err))
    }
}
