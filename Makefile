# Spacesaver - NASA APOD Screen Saver for macOS
#
# Build targets:
#   make build     - Build the screen saver
#   make install   - Install to ~/Library/Screen Savers
#   make uninstall - Remove from ~/Library/Screen Savers
#   make clean     - Clean build artifacts
#   make test      - Run Rust tests
#

.PHONY: all build install uninstall clean test rust-build swift-build bundle

# Configuration
PROJECT_ROOT := $(shell pwd)
RUST_DIR := $(PROJECT_ROOT)/spacesaver-core
SWIFT_DIR := $(PROJECT_ROOT)/swift-wrapper/Spacesaver
BUILD_DIR := $(PROJECT_ROOT)/build
BUNDLE_NAME := Spacesaver.saver
INSTALL_DIR := $(HOME)/Library/Screen Savers

# Detect architecture
ARCH := $(shell uname -m)
ifeq ($(ARCH),arm64)
    RUST_TARGET := aarch64-apple-darwin
    SWIFT_TARGET := arm64-apple-macosx10.15
else
    RUST_TARGET := x86_64-apple-darwin
    SWIFT_TARGET := x86_64-apple-macosx10.15
endif

# SDK path
SDK_PATH := $(shell xcrun --show-sdk-path 2>/dev/null || echo "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk")

all: build

# Build everything
build: rust-build swift-build bundle
	@echo "Build complete: $(BUILD_DIR)/$(BUNDLE_NAME)"

# Build the Rust core library
rust-build:
	@echo "Building Rust library for $(RUST_TARGET)..."
	@mkdir -p $(BUILD_DIR)
	cd $(RUST_DIR) && cargo build --release --target $(RUST_TARGET)
	cp $(RUST_DIR)/target/$(RUST_TARGET)/release/libspacesaver_core.a $(BUILD_DIR)/
	@if [ -f $(RUST_DIR)/include/spacesaver.h ]; then \
		cp $(RUST_DIR)/include/spacesaver.h $(SWIFT_DIR)/; \
	else \
		echo "Warning: Header file not generated. Run 'cargo build' in spacesaver-core first."; \
	fi

# Build the Swift wrapper
swift-build: rust-build
	@echo "Building Swift wrapper..."
	swiftc \
		-emit-library \
		-emit-module \
		-module-name Spacesaver \
		-target $(SWIFT_TARGET) \
		-sdk $(SDK_PATH) \
		-import-objc-header $(SWIFT_DIR)/Spacesaver-Bridging-Header.h \
		-I $(SWIFT_DIR) \
		-L $(BUILD_DIR) \
		-lspacesaver_core \
		-framework ScreenSaver \
		-framework AppKit \
		-framework CoreFoundation \
		-framework Security \
		-framework SystemConfiguration \
		-Xlinker -install_name -Xlinker @loader_path/Spacesaver \
		-o $(BUILD_DIR)/libSpacesaver.dylib \
		$(SWIFT_DIR)/SpacesaverView.swift \
		$(SWIFT_DIR)/ConfigureSheetController.swift

# Create the .saver bundle
bundle: swift-build
	@echo "Creating screen saver bundle..."
	@rm -rf $(BUILD_DIR)/$(BUNDLE_NAME)
	@mkdir -p $(BUILD_DIR)/$(BUNDLE_NAME)/Contents/MacOS
	@mkdir -p $(BUILD_DIR)/$(BUNDLE_NAME)/Contents/Resources
	@mkdir -p $(BUILD_DIR)/$(BUNDLE_NAME)/Contents/Frameworks
	cp $(SWIFT_DIR)/Info.plist $(BUILD_DIR)/$(BUNDLE_NAME)/Contents/
	cp $(BUILD_DIR)/libSpacesaver.dylib $(BUILD_DIR)/$(BUNDLE_NAME)/Contents/MacOS/Spacesaver
	@echo "Bundle created: $(BUILD_DIR)/$(BUNDLE_NAME)"

# Install the screen saver
install: build
	@echo "Installing to $(INSTALL_DIR)..."
	@mkdir -p "$(INSTALL_DIR)"
	cp -r $(BUILD_DIR)/$(BUNDLE_NAME) "$(INSTALL_DIR)/"
	@echo "Installed! Open System Preferences > Screen Saver to select Spacesaver."

# Uninstall the screen saver
uninstall:
	@echo "Removing $(INSTALL_DIR)/$(BUNDLE_NAME)..."
	rm -rf "$(INSTALL_DIR)/$(BUNDLE_NAME)"
	@echo "Uninstalled."

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)
	cd $(RUST_DIR) && cargo clean

# Run Rust tests
test:
	cd $(RUST_DIR) && cargo test

# Development: watch for changes and rebuild
watch:
	@echo "Watching for changes..."
	@while true; do \
		find $(RUST_DIR)/src $(SWIFT_DIR) -name '*.rs' -o -name '*.swift' | \
		entr -d make build; \
	done

# Build universal binary (both arm64 and x86_64)
universal:
	@echo "Building universal binary..."
	@mkdir -p $(BUILD_DIR)/universal
	# Build for arm64
	cd $(RUST_DIR) && cargo build --release --target aarch64-apple-darwin
	# Build for x86_64
	cd $(RUST_DIR) && cargo build --release --target x86_64-apple-darwin
	# Create universal library
	lipo -create \
		$(RUST_DIR)/target/aarch64-apple-darwin/release/libspacesaver_core.a \
		$(RUST_DIR)/target/x86_64-apple-darwin/release/libspacesaver_core.a \
		-output $(BUILD_DIR)/universal/libspacesaver_core.a
	@echo "Universal library created at $(BUILD_DIR)/universal/libspacesaver_core.a"

# Help
help:
	@echo "Spacesaver - NASA APOD Screen Saver"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build     - Build the screen saver (default)"
	@echo "  install   - Install to ~/Library/Screen Savers"
	@echo "  uninstall - Remove from ~/Library/Screen Savers"
	@echo "  clean     - Clean build artifacts"
	@echo "  test      - Run Rust tests"
	@echo "  universal - Build universal binary for both arm64 and x86_64"
	@echo "  help      - Show this help message"
