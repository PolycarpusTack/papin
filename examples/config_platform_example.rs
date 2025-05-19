// examples/config_platform_example.rs
//
// Example of using the cross-platform configuration management system

use mcp_common::config::{
    ConfigManager, ConfigManagerBuilder, ConfigVersion,
    get_config_manager, migrate_to_v1_1_0,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Method 1: Use the global config manager
    {
        println!("==== Using global config manager ====");
        
        // Get a config manager for application settings
        let config = get_config_manager("app_settings.json")?;
        
        // Set some values
        config.set("theme", "dark")?;
        config.set("font_size", 14)?;
        
        // Read values
        let theme = config.get::<String>("theme").unwrap_or_default();
        let font_size = config.get::<i32>("font_size").unwrap_or(12);
        
        println!("Theme: {}", theme);
        println!("Font size: {}", font_size);
        
        // Get a platform-specific default value
        let minimize_to_tray = config.get::<bool>("app.minimize_to_tray").unwrap_or(false);
        println!("Minimize to tray: {} (platform default)", minimize_to_tray);
        
        // Save configuration (not actually needed since auto-save is enabled by default)
        config.save()?;
    }
    
    // Method 2: Create a custom config manager with migrations
    {
        println!("\n==== Using custom config manager with migrations ====");
        
        // Create a builder
        let builder = ConfigManagerBuilder::new("user_preferences.json")
            .auto_save(true)
            .with_migration(ConfigVersion::new(1, 1, 0), migrate_to_v1_1_0);
        
        // Build the manager
        let config = builder.build()?;
        
        // Set some values
        config.set("color_scheme", "system")?;
        config.set("show_notifications", true)?;
        
        // Read values
        let color_scheme = config.get::<String>("color_scheme").unwrap_or_default();
        let show_notifications = config.get::<bool>("show_notifications").unwrap_or(false);
        
        println!("Color scheme: {}", color_scheme);
        println!("Show notifications: {}", show_notifications);
        
        // Get all config keys
        let keys = config.keys();
        println!("All config keys: {:?}", keys);
    }
    
    // Method 3: Create a manager directly for temporary use
    {
        println!("\n==== Using temporary config manager ====");
        
        // Create a manager directly
        let config = ConfigManager::new("temp_settings.json")?;
        
        // Set some values without saving
        config.set_auto_save(false);
        config.set("temp_setting", "This will not be saved automatically")?;
        
        // Read the value
        let temp_setting = config.get::<String>("temp_setting").unwrap_or_default();
        println!("Temporary setting: {}", temp_setting);
        
        // To save manually
        config.save()?;
        println!("Configuration saved manually");
    }
    
    println!("\nConfiguration files are stored in: {:?}", get_platform_config_dir());
    
    Ok(())
}

// Helper function to get the configuration directory
fn get_platform_config_dir() -> std::path::PathBuf {
    use mcp_common::platform::fs::platform_fs;
    
    let fs = platform_fs();
    fs.app_data_dir("Papin").unwrap_or_else(|_| {
        std::path::PathBuf::from("Could not determine config directory")
    })
}
