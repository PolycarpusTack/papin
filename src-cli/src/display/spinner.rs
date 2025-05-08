use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Spinner handle
pub struct SpinnerHandle {
    bar: ProgressBar,
}

impl SpinnerHandle {
    /// Set a new message for the spinner
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }
    
    /// Finish with a success message
    pub fn success(self, message: &str) {
        self.bar.finish_with_message(format!("✓ {}", message));
    }
    
    /// Finish with an error message
    pub fn error(self, message: &str) {
        self.bar.finish_with_message(format!("✗ {}", message));
    }
    
    /// Finish with a warning message
    pub fn warning(self, message: &str) {
        self.bar.finish_with_message(format!("⚠ {}", message));
    }
    
    /// Finish with an info message
    pub fn info(self, message: &str) {
        self.bar.finish_with_message(format!("ℹ {}", message));
    }
    
    /// Abandon the spinner (finish without message)
    pub fn abandon(self) {
        self.bar.finish_and_clear();
    }
}

/// Show a spinner with default message "Processing..."
pub fn show_spinner() -> SpinnerHandle {
    show_spinner_with_message("Processing...")
}

/// Show a spinner with a custom message
pub fn show_spinner_with_message(message: &str) -> SpinnerHandle {
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        .template("{spinner} {msg}")
        .unwrap();
    
    let bar = ProgressBar::new_spinner();
    bar.set_style(spinner_style);
    bar.set_message(message.to_string());
    bar.enable_steady_tick(Duration::from_millis(100));
    
    SpinnerHandle { bar }
}
