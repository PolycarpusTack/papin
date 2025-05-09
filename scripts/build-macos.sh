#!/bin/bash
# Build script for macOS

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting MCP Client build for macOS...${NC}"

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

# Build for macOS
echo -e "${YELLOW}Building for macOS (.app and .dmg)...${NC}"
cargo tauri build

# Move all built packages to the dist directory
echo -e "${YELLOW}Moving packages to dist directory...${NC}"
mkdir -p "$PROJECT_ROOT/dist"

# Find and move the built packages
find "$PROJECT_ROOT/src-tauri/target/release/bundle/dmg" -name "*.dmg" | while read file; do
  cp "$file" "$PROJECT_ROOT/dist/"
  echo -e "${GREEN}Copied $(basename "$file") to dist directory${NC}"
done

# For notarization (would be done in production environment)
echo -e "${YELLOW}NOTE: For distribution on macOS, the app should be notarized.${NC}"
echo -e "${YELLOW}This requires an Apple Developer account and is not included in this script.${NC}"

# Generate checksums
echo -e "${YELLOW}Generating checksums...${NC}"
cd "$PROJECT_ROOT/dist"
shasum -a 256 *.dmg > checksums.sha256
echo -e "${GREEN}Checksums generated in dist/checksums.sha256${NC}"

echo -e "${GREEN}Build completed successfully!${NC}"
echo -e "${BLUE}Packages available in the dist directory:${NC}"
ls -la "$PROJECT_ROOT/dist"
