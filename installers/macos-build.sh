#!/bin/bash

# Build script for macOS DMG installers
# Supports Universal, Intel, and Apple Silicon builds

set -e

# Get the project directory
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

echo "Building MCP Client for macOS..."

# Ensure required tools are installed
check_tools() {
  for tool in $@; do
    if ! command -v $tool &> /dev/null; then
      echo "Error: $tool is not installed. Please install it first."
      exit 1
    fi
  done
}

check_tools cargo npm rustup

# Update dependencies
echo "Updating dependencies..."
npm install
cargo update

# Add Apple Silicon target if not already added
if ! rustup target list | grep -q "aarch64-apple-darwin"; then
  echo "Adding aarch64-apple-darwin target..."
  rustup target add aarch64-apple-darwin
fi

# Build the application
echo "Building application..."
npm run build

# Build for Intel x86_64
echo "Building for Intel (x86_64)..."
cargo tauri build --target x86_64-apple-darwin
echo "Intel build completed successfully!"

# Build for Apple Silicon
echo "Building for Apple Silicon (aarch64)..."
cargo tauri build --target aarch64-apple-darwin
echo "Apple Silicon build completed successfully!"

# Build Universal Binary
echo "Building Universal Binary..."
cargo tauri build --target universal-apple-darwin
echo "Universal Binary build completed successfully!"

# Move built packages to installers directory
echo "Moving packages to installers directory..."
mkdir -p "$PROJECT_DIR/installers/macos"

# Copy DMG files to installers directory
cp "$PROJECT_DIR/src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/"*.dmg "$PROJECT_DIR/installers/macos/MCP-Client-Intel.dmg"
cp "$PROJECT_DIR/src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/"*.dmg "$PROJECT_DIR/installers/macos/MCP-Client-AppleSilicon.dmg"
cp "$PROJECT_DIR/src-tauri/target/universal-apple-darwin/release/bundle/dmg/"*.dmg "$PROJECT_DIR/installers/macos/MCP-Client-Universal.dmg"

# Generate checksums
echo "Generating checksums..."
cd "$PROJECT_DIR/installers/macos"
shasum -a 256 *.dmg > checksums.txt

echo "macOS builds completed successfully!"
echo "Packages are available in the '$PROJECT_DIR/installers/macos' directory."