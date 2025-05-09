# Windows Build Script for MCP Client
# Generates MSI installer and portable EXE

# Ensure script stops on errors
$ErrorActionPreference = "Stop"

Write-Host "Building MCP Client for Windows..." -ForegroundColor Green

# Get project directory
$ProjectDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $ProjectDir

# Check for required tools
function Check-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

if (-not (Check-Command "cargo")) {
    Write-Host "Error: Cargo is not installed. Please install Rust and Cargo first." -ForegroundColor Red
    exit 1
}

if (-not (Check-Command "npm")) {
    Write-Host "Error: NPM is not installed. Please install Node.js first." -ForegroundColor Red
    exit 1
}

# Update dependencies
Write-Host "Updating dependencies..." -ForegroundColor Cyan
npm install
cargo update

# Build the application
Write-Host "Building application..." -ForegroundColor Cyan
npm run build

# Build MSI installer
Write-Host "Building MSI installer..." -ForegroundColor Cyan
cargo tauri build --target msi
Write-Host "MSI installer built successfully!" -ForegroundColor Green

# Build portable version
Write-Host "Building portable EXE..." -ForegroundColor Cyan
cargo tauri build
Write-Host "Portable EXE built successfully!" -ForegroundColor Green

# Create installers directory if it doesn't exist
$InstallerDir = Join-Path $ProjectDir "installers\windows"
if (-not (Test-Path $InstallerDir)) {
    New-Item -ItemType Directory -Path $InstallerDir -Force | Out-Null
}

# Move built packages to installers directory
Write-Host "Moving packages to installers directory..." -ForegroundColor Cyan
$MsiPath = Get-ChildItem -Path "$ProjectDir\src-tauri\target\release\bundle\msi\*.msi" | Select-Object -First 1
$ExePath = "$ProjectDir\src-tauri\target\release\mcp-client.exe"

Copy-Item $MsiPath -Destination $InstallerDir
Copy-Item $ExePath -Destination "$InstallerDir\MCP-Client-Portable.exe"

# Generate checksums
Write-Host "Generating checksums..." -ForegroundColor Cyan
Set-Location $InstallerDir
Get-FileHash -Algorithm SHA256 *.msi, *.exe | ForEach-Object {
    "$($_.Hash) $($_.Path.Split('\')[-1])"
} | Out-File -FilePath "checksums.txt" -Encoding utf8

Write-Host "Windows builds completed successfully!" -ForegroundColor Green
Write-Host "Packages are available in the '$InstallerDir' directory." -ForegroundColor Cyan