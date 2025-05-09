#!/bin/bash

# Build script for Linux distribution packages
# Supports: DEB, RPM, and AppImage

set -e

# Get the project directory
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

echo "Building MCP Client for Linux distributions..."

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

# Build the application
echo "Building application..."
npm run build

# Build for specific Linux targets
echo "Building DEB package..."
cargo tauri build --target deb
echo "DEB package built successfully!"

echo "Building RPM package..."
cargo tauri build --target rpm
echo "RPM package built successfully!"

echo "Building AppImage..."
cargo tauri build --target appimage
echo "AppImage built successfully!"

# Move built packages to installers directory
echo "Moving packages to installers directory..."
mkdir -p "$PROJECT_DIR/installers/linux"
mv "$PROJECT_DIR/src-tauri/target/release/bundle/deb/"*.deb "$PROJECT_DIR/installers/linux/"
mv "$PROJECT_DIR/src-tauri/target/release/bundle/rpm/"*.rpm "$PROJECT_DIR/installers/linux/"
mv "$PROJECT_DIR/src-tauri/target/release/bundle/appimage/"*.AppImage "$PROJECT_DIR/installers/linux/"

# Generate checksums
echo "Generating checksums..."
cd "$PROJECT_DIR/installers/linux"
sha256sum *.deb *.rpm *.AppImage > checksums.txt

echo "Linux builds completed successfully!"
echo "Packages are available in the '$PROJECT_DIR/installers/linux' directory."