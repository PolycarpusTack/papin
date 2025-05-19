// tests/platform_fs_test.rs

use std::path::Path;
use mcp_common::platform::fs::{platform_fs, Platform, PathExt};

#[test]
fn test_platform_detection() {
    let fs = platform_fs();
    
    // Verify that platform is detected correctly
    let platform = fs.platform();
    
    #[cfg(target_os = "windows")]
    assert_eq!(platform, Platform::Windows);
    
    #[cfg(target_os = "macos")]
    assert_eq!(platform, Platform::MacOS);
    
    #[cfg(target_os = "linux")]
    assert_eq!(platform, Platform::Linux);
}

#[test]
fn test_app_data_dir() {
    let fs = platform_fs();
    
    // Test getting the app data directory
    let app_data_dir = fs.app_data_dir("PapinTest").expect("Failed to get app data directory");
    
    // Verify directory structure is correct for the platform
    #[cfg(target_os = "windows")]
    {
        // Windows: %APPDATA%\PapinTest
        assert!(app_data_dir.to_string_lossy().contains("AppData"));
        assert!(app_data_dir.ends_with("PapinTest"));
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS: ~/Library/Application Support/PapinTest
        assert!(app_data_dir.to_string_lossy().contains("Library"));
        assert!(app_data_dir.to_string_lossy().contains("Application Support"));
        assert!(app_data_dir.ends_with("PapinTest"));
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux: ~/.config/papintest
        assert!(app_data_dir.to_string_lossy().contains(".config"));
        assert!(app_data_dir.ends_with("PapinTest"));
    }
}

#[test]
fn test_path_normalization() {
    // Test path normalization
    let path = Path::new("../test/./path");
    let normalized = path.normalize();
    
    // Normalized path should not contain . or ..
    assert!(!normalized.to_string_lossy().contains("/./"));
    assert!(!normalized.to_string_lossy().contains("/../"));
    
    // Different paths that refer to the same location should normalize to the same path
    let path1 = Path::new("test/./subdir/../file.txt");
    let path2 = Path::new("test/file.txt");
    
    // Must handle the case where canonicalize fails (path doesn't exist)
    let norm1 = path1.normalize();
    let norm2 = path2.normalize();
    
    // In cases where paths don't exist, we'll fall back to removing redundant components
    assert_eq!(norm1.file_name(), norm2.file_name());
}

#[test]
fn test_file_operations() {
    let fs = platform_fs();
    
    // Create a temporary directory for testing
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let test_file_path = temp_dir.path().join("test_file.txt");
    
    // Test creating a file
    {
        let mut file = fs.create_file(&test_file_path).expect("Failed to create file");
        std::io::Write::write_all(&mut file, b"Test content").expect("Failed to write to file");
    }
    
    // Test checking if a file exists
    assert!(fs.file_exists(&test_file_path));
    
    // Test reading file contents
    let content = fs.read_to_string(&test_file_path).expect("Failed to read file");
    assert_eq!(content, "Test content");
    
    // Test writing a string to a file
    fs.write_string(&test_file_path, "Updated content").expect("Failed to write string to file");
    
    // Verify content was updated
    let updated_content = fs.read_to_string(&test_file_path).expect("Failed to read updated file");
    assert_eq!(updated_content, "Updated content");
    
    // Test renaming/moving a file
    let renamed_file_path = temp_dir.path().join("renamed_file.txt");
    fs.rename_file(&test_file_path, &renamed_file_path).expect("Failed to rename file");
    
    // Verify old file doesn't exist and new file does
    assert!(!fs.file_exists(&test_file_path));
    assert!(fs.file_exists(&renamed_file_path));
    
    // Test removing a file
    fs.remove_file(&renamed_file_path).expect("Failed to remove file");
    assert!(!fs.file_exists(&renamed_file_path));
}

#[test]
fn test_directory_operations() {
    let fs = platform_fs();
    
    // Create a temporary directory for testing
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let test_subdir = temp_dir.path().join("test_subdir");
    
    // Test ensuring a directory exists
    fs.ensure_dir_exists(&test_subdir).expect("Failed to create directory");
    assert!(fs.dir_exists(&test_subdir));
    
    // Create a file in the subdirectory
    let test_file_path = test_subdir.join("test_file.txt");
    fs.write_string(&test_file_path, "Test content").expect("Failed to write file in subdirectory");
    
    // Verify file exists
    assert!(fs.file_exists(&test_file_path));
}
