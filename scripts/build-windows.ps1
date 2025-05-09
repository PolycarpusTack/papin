# Build script for Windows

# Colors for output
$Red = [System.ConsoleColor]::Red
$Green = [System.ConsoleColor]::Green
$Blue = [System.ConsoleColor]::Blue
$Yellow = [System.ConsoleColor]::Yellow

function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    else {
        $input | Write-Output
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

Write-ColorOutput $Blue "Starting MCP Client build for Windows..."

# Make sure we're in the project root
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = (Get-Item $scriptPath).Parent.FullName
Set-Location $projectRoot

# Ensure dependencies are installed
Write-ColorOutput $Yellow "Checking and installing dependencies..."
if (Test-Path "package.json") {
    npm install
}
else {
    Write-ColorOutput $Red "Error: package.json not found. Are you in the right directory?"
    exit 1
}

# Build the frontend
Write-ColorOutput $Yellow "Building frontend..."
npm run build

# Check if the build was successful
if ($LASTEXITCODE -ne 0) {
    Write-ColorOutput $Red "Frontend build failed!"
    exit 1
}
Write-ColorOutput $Green "Frontend build successful!"

# Build for Windows
Write-ColorOutput $Yellow "Building for Windows (MSI)..."
cargo tauri build

# Move built package to the dist directory
Write-ColorOutput $Yellow "Moving package to dist directory..."
$distDir = Join-Path $projectRoot "dist"
if (-Not (Test-Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir | Out-Null
}

# Find and move the built MSI package
$msiPackage = Get-ChildItem -Path (Join-Path $projectRoot "src-tauri\target\release\bundle\msi") -Filter "*.msi" | Select-Object -First 1
if ($msiPackage) {
    Copy-Item $msiPackage.FullName -Destination $distDir
    Write-ColorOutput $Green "Copied $($msiPackage.Name) to dist directory"
}
else {
    Write-ColorOutput $Red "No MSI package found!"
    exit 1
}

# Generate checksums
Write-ColorOutput $Yellow "Generating checksums..."
Set-Location $distDir
Get-FileHash -Algorithm SHA256 *.msi | ForEach-Object { "$($_.Hash) $($_.Path)" } | Out-File -FilePath "checksums.sha256" -Encoding ascii
Write-ColorOutput $Green "Checksums generated in dist/checksums.sha256"

Write-ColorOutput $Green "Build completed successfully!"
Write-ColorOutput $Blue "Packages available in the dist directory:"
Get-ChildItem $distDir | Format-Table Name, Length -AutoSize
