# Cross-Platform Configuration Management

This document explains how to use the cross-platform configuration management system implemented in `src-common/src/config/platform.rs`.

## Overview

The configuration system is designed to handle application settings in a platform-appropriate way across Windows, macOS, and Linux. It provides:

1. **Platform-Specific Storage Locations**:
   - Windows: `%APPDATA%\Papin`
   - macOS: `~/Library/Application Support/Papin`
   - Linux: `~/.config/papin`

2. **Platform-Specific Defaults**:
   - Each platform has its own default values for common settings.
   - For example, the default theme is "light" on Windows, "system" on macOS, and "dark" on Linux.

3. **Migration Support**:
   - Configuration files are versioned.
   - Migration handlers can be registered to automatically update older configuration files.

4. **Unified API**:
   - Simple get/set methods for reading and writing configuration values.
   - Strongly typed with automatic serialization/deserialization.

5. **Robust Error Handling**:
   - Detailed error types for configuration-related failures.
   - Graceful fallbacks when config files can't be accessed.

## Basic Usage

### Getting a Configuration Manager

The simplest way to use the configuration system is to get a global configuration manager:

```rust
use mcp_common::config::get_config_manager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config_manager("app_settings.json")?;
    
    // Now you can use the config manager
    config.set("theme", "dark")?;
    
    let theme = config.get::<String>("theme").unwrap_or_default();
    println!("Theme: {}", theme);
    
    Ok(())
}
```

### Reading and Writing Configuration

The configuration manager provides methods for reading and writing values:

```rust
// Setting values
config.set("theme", "dark")?;
config.set("font_size", 14)?;
config.set("show_sidebar", true)?;

// Setting nested values
config.set("appearance.colors.primary", "#3498db")?;

// Getting values
let theme: String = config.get("theme").unwrap_or_default();
let font_size: i32 = config.get("font_size").unwrap_or(12);
let show_sidebar: bool = config.get("show_sidebar").unwrap_or(false);

// Getting values with default
let line_spacing = config.get_or("editor.line_spacing", 1.5);

// Getting nested values
let primary_color: String = config.get("appearance.colors.primary").unwrap_or("#000000".to_string());
```

### Platform-Specific Defaults

The configuration system provides platform-specific default values:

```rust
// This will return different values depending on the platform
let theme = config.get::<String>("appearance.theme").unwrap_or_default();
let minimize_to_tray = config.get::<bool>("app.minimize_to_tray").unwrap_or(false);
```

The following default values are provided:

| Setting | Windows | macOS | Linux |
|---------|---------|-------|-------|
| appearance.theme | "light" | "system" | "dark" |
| appearance.use_system_theme | true | true | true |
| app.minimize_to_tray | true | false | true |
| app.start_on_boot | false | false | false |
| networking.proxy.use_system | true | true | true |
| paths.downloads | User's Downloads folder | User's Downloads folder | User's Downloads folder |

You can add more platform-specific defaults by modifying the platform defaults classes in `platform.rs`.

### Advanced Usage: Custom Configuration Manager

For more advanced use cases, you can create a custom configuration manager with specific settings:

```rust
use mcp_common::config::{ConfigManagerBuilder, ConfigVersion, migrate_to_v1_1_0};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder
    let builder = ConfigManagerBuilder::new("user_preferences.json")
        .auto_save(true)
        .with_migration(ConfigVersion::new(1, 1, 0), migrate_to_v1_1_0);
    
    // Build the manager
    let config = builder.build()?;
    
    // Now you can use the config manager
    config.set("color_scheme", "system")?;
    
    Ok(())
}
```

### Configuration Migrations

The configuration system supports versioned migrations to handle changes to the configuration structure:

```rust
use mcp_common::config::{ConfigManager, ConfigVersion, ConfigError};

// Define a migration handler
fn migrate_to_v1_1_0(manager: &mut ConfigManager, from_version: &ConfigVersion) -> Result<(), ConfigError> {
    // Rename a configuration key
    if let Some(value) = manager.get_json("old_key") {
        manager.set_json("new_key", value)?;
        manager.remove("old_key")?;
    }
    
    // Change the format of a value
    if let Some(old_value) = manager.get::<String>("some_setting") {
        let new_value = format!("updated:{}", old_value);
        manager.set("some_setting", new_value)?;
    }
    
    Ok(())
}

// Register the migration handler
let mut builder = ConfigManagerBuilder::new("config.json");
builder = builder.with_migration(ConfigVersion::new(1, 1, 0), migrate_to_v1_1_0);
let config = builder.build()?;
```

## Error Handling

The configuration system provides detailed error types for handling failures:

```rust
use mcp_common::config::{get_config_manager, ConfigError};

fn main() {
    let result = get_config_manager("app_settings.json");
    
    match result {
        Ok(config) => {
            // Use the config manager
        },
        Err(err) => {
            match err {
                ConfigError::Io(io_err) => {
                    eprintln!("IO error: {}", io_err);
                },
                ConfigError::FileSystem(fs_err) => {
                    eprintln!("File system error: {}", fs_err);
                },
                ConfigError::Json(json_err) => {
                    eprintln!("JSON parsing error: {}", json_err);
                },
                ConfigError::NotFound(file) => {
                    eprintln!("Config file not found: {}", file);
                },
                ConfigError::Incompatible(exp_major, found_major, found_minor, found_patch) => {
                    eprintln!("Config version incompatible: expected v{}.x.x, found v{}.{}.{}", 
                            exp_major, found_major, found_minor, found_patch);
                },
                ConfigError::Migration(msg) => {
                    eprintln!("Migration error: {}", msg);
                },
                ConfigError::Invalid(msg) => {
                    eprintln!("Invalid config: {}", msg);
                },
            }
        },
    }
}
```

## Thread Safety

The configuration system is designed to be thread-safe:

- The `ConfigManager` uses `RwLock` internally to allow concurrent readers.
- Global configuration managers are stored in an `Arc` for safe sharing between threads.
- All methods take `&self` rather than `&mut self` to allow shared usage.

Example of using a config manager from multiple threads:

```rust
use std::sync::Arc;
use std::thread;
use mcp_common::config::get_config_manager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config_manager("app_settings.json")?;
    
    let mut handles = vec![];
    
    // Spawn threads that read from the config
    for i in 0..5 {
        let config_clone = config.clone();
        let handle = thread::spawn(move || {
            let value = config_clone.get::<String>("setting").unwrap_or_default();
            println!("Thread {}: Read value: {}", i, value);
        });
        handles.push(handle);
    }
    
    // Spawn threads that write to the config
    for i in 0..5 {
        let config_clone = config.clone();
        let handle = thread::spawn(move || {
            let key = format!("thread_{}_setting", i);
            let value = format!("Value from thread {}", i);
            config_clone.set(&key, value).unwrap();
            println!("Thread {}: Wrote value", i);
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    Ok(())
}
```

## Implementation Details

### File Format

Configuration files are stored as JSON with a special `_metadata` field that contains versioning and other metadata:

```json
{
  "_metadata": {
    "version": {
      "major": 1,
      "minor": 0,
      "patch": 0
    },
    "last_modified": "2023-05-01T12:34:56Z",
    "platform": "windows",
    "app_version": "1.0.0"
  },
  "theme": "dark",
  "font_size": 14,
  "show_sidebar": true
}
```

### Performance Considerations

- Configuration values are cached in memory for fast access.
- By default, changes are immediately saved to disk, but this can be disabled.
- The configuration system is designed to be lightweight and efficient.

## Best Practices

1. **Use Dot Notation for Hierarchical Settings**:
   ```rust
   // Good
   config.set("appearance.theme", "dark")?;
   config.set("appearance.colors.primary", "#3498db")?;
   
   // Avoid
   config.set("appearance_theme", "dark")?;
   config.set("appearance_colors_primary", "#3498db")?;
   ```

2. **Group Related Settings**:
   ```rust
   // Good
   config.set("editor.font_family", "Consolas")?;
   config.set("editor.font_size", 14)?;
   
   // Avoid
   config.set("editor_font_family", "Consolas")?;
   config.set("font_size_for_editor", 14)?;
   ```

3. **Handle Missing Values Gracefully**:
   ```rust
   // Good
   let font_size = config.get::<i32>("editor.font_size").unwrap_or(12);
   
   // Avoid
   let font_size = config.get::<i32>("editor.font_size").unwrap(); // May panic
   ```

4. **Use Strong Typing**:
   ```rust
   // Good
   let font_size: i32 = config.get("editor.font_size").unwrap_or(12);
   
   // Avoid
   let font_size = config.get_json("editor.font_size").unwrap().as_i64().unwrap() as i32;
   ```

5. **Register Migrations for Backwards Compatibility**:
   ```rust
   // Good
   builder = builder.with_migration(ConfigVersion::new(1, 1, 0), migrate_to_v1_1_0);
   
   // Avoid
   // (Not handling old config formats)
   ```

## Conclusion

The cross-platform configuration management system provides a robust, efficient, and user-friendly way to handle application settings across different platforms. It ensures a consistent experience while respecting platform conventions and preferences.
