use console::style;
use std::io::{self, Write};

/// Print an informational message
pub fn print_info(message: &str) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{} {}", style("[INFO]").cyan(), message);
}

/// Print a success message
pub fn print_success(message: &str) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{} {}", style("[SUCCESS]").green(), message);
}

/// Print a warning message
pub fn print_warning(message: &str) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{} {}", style("[WARNING]").yellow(), message);
}

/// Print an error message
pub fn print_error(message: &str) {
    let stderr = io::stderr();
    let mut handle = stderr.lock();
    let _ = writeln!(handle, "{} {}", style("[ERROR]").red().bold(), message);
}
