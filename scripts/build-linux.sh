#!/bin/bash
# Build script for Linux distributions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting MCP Client build for Linux distributions...${NC}"

# Make sure we're in the project root
cd "$(dirname "$0")/.."
PROJECT_ROOT=$(pwd)

# Ensure dependencies are installed
echo -e "${YELLOW}Checking and installing dependencies...${NC}"
if [ -f "package.json" ]; then
  npm install
else
  echo -e "${RED}Error: package.json not found. Are you in the right directory?${NC}"
  exit 1
fi

# Build the frontend
echo -e "${YELLOW}Building frontend...${NC}"
npm run build

# Check if the build was successful
if [ $? -ne 0 ]; then
  echo -e "${RED}Frontend build failed!${NC}"
  exit 1
fi
echo -e "${GREEN}Frontend build successful!${NC}"

# Build for various Linux targets
echo -e "${YELLOW}Building for Debian...${NC}"
cargo tauri build --target deb

echo -e "${YELLOW}Building for RPM-based distributions...${NC}"
cargo tauri build --target rpm

echo -e "${YELLOW}Building AppImage...${NC}"
cargo tauri build --target appimage

# Move all built packages to the dist directory
echo -e "${YELLOW}Moving packages to dist directory...${NC}"
mkdir -p "$PROJECT_ROOT/dist"

# Find and move the built packages
find "$PROJECT_ROOT/src-tauri/target/release/bundle" -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" | while read file; do
  cp "$file" "$PROJECT_ROOT/dist/"
  echo -e "${GREEN}Copied $(basename "$file") to dist directory${NC}"
done

# Generate checksums
echo -e "${YELLOW}Generating checksums...${NC}"
cd "$PROJECT_ROOT/dist"
sha256sum *.deb *.rpm *.AppImage > checksums.sha256
echo -e "${GREEN}Checksums generated in dist/checksums.sha256${NC}"

echo -e "${GREEN}Build completed successfully!${NC}"
echo -e "${BLUE}Packages available in the dist directory:${NC}"
ls -la "$PROJECT_ROOT/dist"
