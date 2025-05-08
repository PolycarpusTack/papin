.PHONY: all clean build-common build-cli build-tui build-gui build-all install-cli install-tui install-gui install-all

# Variables
CARGO := cargo
CARGO_BUILD_ARGS := --release
RUST_BACKTRACE := 1
INSTALL_BIN_DIR := $(HOME)/.local/bin

# Default target
all: build-all

# Clean all targets
clean:
	@echo "Cleaning all targets..."
	$(CARGO) clean
	cd src-frontend && npm run clean || echo "No clean script in frontend"

# Build common library
build-common:
	@echo "Building common library..."
	cd src-common && $(CARGO) build $(CARGO_BUILD_ARGS)

# Build CLI
build-cli: build-common
	@echo "Building CLI..."
	cd src-cli && $(CARGO) build $(CARGO_BUILD_ARGS)

# Build TUI
build-tui: build-common
	@echo "Building TUI..."
	cd src-tui && $(CARGO) build $(CARGO_BUILD_ARGS)

# Build GUI (Tauri app)
build-gui: build-common
	@echo "Building GUI..."
	@echo "Installing frontend dependencies..."
	cd src-frontend && npm install
	@echo "Building Tauri app..."
	$(CARGO) tauri build

# Build all components
build-all: build-cli build-tui build-gui

# Install CLI
install-cli: build-cli
	@echo "Installing CLI..."
	mkdir -p $(INSTALL_BIN_DIR)
	cp target/release/mcp-cli $(INSTALL_BIN_DIR)/mcp
	@echo "CLI installed to $(INSTALL_BIN_DIR)/mcp"

# Install TUI
install-tui: build-tui
	@echo "Installing TUI..."
	mkdir -p $(INSTALL_BIN_DIR)
	cp target/release/mcp-tui $(INSTALL_BIN_DIR)/mcp-tui
	@echo "TUI installed to $(INSTALL_BIN_DIR)/mcp-tui"

# Install GUI
install-gui: build-gui
	@echo "Installing GUI..."
	@echo "To install the GUI, use the generated .deb or .AppImage file in src-tauri/target/release."

# Install all components
install-all: install-cli install-tui install-gui

# Run CLI
run-cli:
	@echo "Running CLI..."
	cd src-cli && $(CARGO) run

# Run TUI
run-tui:
	@echo "Running TUI..."
	cd src-tui && $(CARGO) run

# Run GUI in development mode
run-gui:
	@echo "Running GUI in development mode..."
	cd src-frontend && npm run dev
