# MCP Client Installation Guide

This guide provides detailed instructions for installing the MCP Client on various platforms.

## Windows Installation

### System Requirements

- Windows 10 or later (64-bit)
- 4GB RAM minimum (8GB recommended)
- 1GB free disk space
- Administrator privileges for installation

### Using the MSI Installer

1. Download the latest MSI installer from the [releases page](https://github.com/your-org/mcp-client/releases)
2. Right-click the downloaded file and select "Properties"
3. Check "Unblock" if available, then click "OK"
4. Double-click the installer to start the installation process
5. Follow the on-screen instructions
6. Choose the installation location (default is recommended)
7. Select whether to create desktop shortcuts
8. Click "Install" to begin the installation
9. If prompted by UAC (User Account Control), click "Yes" to allow the installation
10. Once installation is complete, click "Finish"

### Using the Portable Version

1. Download the portable EXE from the [releases page](https://github.com/your-org/mcp-client/releases)
2. Create a folder where you want to keep the application (e.g., `C:\Programs\MCP-Client`)
3. Move the downloaded EXE to this folder
4. (Optional) Create a shortcut on the desktop or taskbar
5. Double-click the EXE to run the application

### Silent Installation

For enterprise deployment, you can install silently using:

```batch
msiexec /i MCP-Client-1.0.0-x64.msi /quiet
```

## macOS Installation

### System Requirements

- macOS 10.15 (Catalina) or later
- Intel or Apple Silicon processor
- 4GB RAM minimum (8GB recommended)
- 1GB free disk space

### Using the DMG Installer

1. Download the latest DMG file from the [releases page](https://github.com/your-org/mcp-client/releases)
2. Double-click the downloaded DMG file to mount it
3. Drag the MCP Client application to the Applications folder
4. Eject the mounted DMG
5. Open your Applications folder and right-click on the MCP Client
6. Select "Open"
7. On first launch, you may see a security warning. Click "Open" to proceed
8. If prompted about allowing system access, follow the on-screen instructions

### Apple Silicon vs Intel Macs

- Universal DMG: Works on both Intel and Apple Silicon Macs (recommended)
- Intel DMG: Optimized for Intel Macs
- Apple Silicon DMG: Optimized for Apple Silicon Macs (M1/M2)

### Homebrew Installation

If you use Homebrew, you can install MCP Client with:

```bash
brew install --cask mcp-client
```

## Linux Installation

### System Requirements

- Ubuntu 20.04 or later, Fedora 34 or later, or equivalent
- X11 or Wayland display server
- 4GB RAM minimum (8GB recommended)
- 1GB free disk space

### Debian/Ubuntu Installation (DEB)

1. Download the latest DEB package from the [releases page](https://github.com/your-org/mcp-client/releases)
2. Open a terminal and navigate to the download location
3. Install using apt:

```bash
sudo apt install ./mcp-client_1.0.0_amd64.deb
```

4. Launch from your applications menu or via terminal:

```bash
mcp-client
```

### Fedora/RHEL Installation (RPM)

1. Download the latest RPM package from the [releases page](https://github.com/your-org/mcp-client/releases)
2. Open a terminal and navigate to the download location
3. Install using dnf:

```bash
sudo dnf install ./mcp-client-1.0.0.x86_64.rpm
```

4. Launch from your applications menu or via terminal:

```bash
mcp-client
```

### AppImage Installation

1. Download the latest AppImage from the [releases page](https://github.com/your-org/mcp-client/releases)
2. Open a terminal and navigate to the download location
3. Make the AppImage executable:

```bash
chmod +x MCP-Client-1.0.0.AppImage
```

4. Run the AppImage:

```bash
./MCP-Client-1.0.0.AppImage
```

5. (Optional) Integrate with your desktop:

```bash
./MCP-Client-1.0.0.AppImage --install
```

## Verifying Installation

After installation, verify that the application is working correctly:

1. Launch the MCP Client
2. You should see the login screen or welcome page
3. Check the version number in Help > About to ensure you have the latest version

## Troubleshooting

### Windows Issues

- **Installation Fails**: Ensure you have administrator privileges and check Windows logs
- **Application Crashes**: Update your graphics drivers and ensure DirectX is up to date
- **Missing DLLs**: Install the latest Visual C++ Redistributable packages

### macOS Issues

- **"App is damaged"**: Right-click the app, choose "Open", then click "Open" in the dialog
- **"Unidentified Developer"**: Go to System Preferences > Security & Privacy and click "Open Anyway"
- **App Won't Open**: Check if your macOS version is supported (10.15+)

### Linux Issues

- **Missing Libraries**: Install required dependencies:

```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.0-37 libgtk-3-0 libsoup2.4-1

# Fedora
sudo dnf install webkit2gtk3 gtk3 libsoup
```

- **AppImage Won't Run**: Check file permissions and ensure FUSE is installed:

```bash
sudo apt install fuse libfuse2   # Debian/Ubuntu
sudo dnf install fuse libfuse    # Fedora
```

## Uninstallation

### Windows

- Use Add/Remove Programs in Control Panel
- Or uninstall via command line: `msiexec /x MCP-Client-1.0.0-x64.msi`

### macOS

- Drag the application from Applications to Trash
- To remove all associated files:

```bash
rm -rf ~/Library/Application\ Support/MCP-Client
```

### Linux

#### Debian/Ubuntu:

```bash
sudo apt remove mcp-client
```

#### Fedora/RHEL:

```bash
sudo dnf remove mcp-client
```

#### AppImage:

- Simply delete the AppImage file
- If you used `--install`, run:

```bash
./MCP-Client-1.0.0.AppImage --remove-appimage-desktop-integration
```

## Enterprise Deployment

### Windows MSI Properties

For enterprise deployment, you can use the following MSI properties:

```
INSTALLDIR - Installation directory
CREATEDESKTOPSHORTCUT - "1" to create a desktop shortcut, "0" to skip
STARTMENUSHORTCUT - "1" to create a start menu shortcut, "0" to skip
AUTOSTARTUP - "1" to start on boot, "0" to skip
```

Example:

```batch
msiexec /i MCP-Client-1.0.0-x64.msi INSTALLDIR="C:\Programs\MCP-Client" CREATEDESKTOPSHORTCUT="1" AUTOSTARTUP="0" /quiet
```

### macOS Deployment

For mass deployment on macOS, consider using:

- Jamf Pro
- Munki
- Apple Remote Desktop

### Linux Deployment

- Consider using your distribution's package management system
- For containerized environments, Docker images are available