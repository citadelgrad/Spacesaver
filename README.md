# Spacesaver

A macOS screen saver that displays stunning NASA Astronomy Picture of the Day (APOD) images.

![macOS](https://img.shields.io/badge/macOS-10.15+-blue)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![Swift](https://img.shields.io/badge/Swift-5.5+-red)
![License](https://img.shields.io/badge/License-MIT-green)

## Features

- **NASA APOD Integration**: Automatically fetches beautiful astronomy images from NASA's Astronomy Picture of the Day API
- **Smart Caching**: Downloads and caches images locally for offline viewing
- **Smooth Transitions**: Elegant crossfade transitions between images
- **Information Overlay**: Displays image title, date, and copyright information
- **Configurable**: Customize settings through the Screen Saver preferences panel
- **Built with Rust**: Core image management written in Rust for performance and reliability
- **Universal Binary Support**: Runs natively on both Intel and Apple Silicon Macs

## Requirements

- macOS 10.15 (Catalina) or later
- Rust 1.70+ (for building from source)
- Xcode Command Line Tools (for building from source)

## Installation

### From Release

1. Download the latest `Spacesaver.saver.zip` from the [Releases](https://github.com/yourusername/Spacesaver/releases) page
2. Unzip and double-click `Spacesaver.saver` to install
3. Open System Preferences > Screen Saver and select "Spacesaver"

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/Spacesaver.git
cd Spacesaver

# Build
make build

# Install
make install
```

## Building

### Prerequisites

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Add the macOS target (if cross-compiling):
   ```bash
   rustup target add aarch64-apple-darwin  # For Apple Silicon
   rustup target add x86_64-apple-darwin   # For Intel
   ```

3. Install Xcode Command Line Tools:
   ```bash
   xcode-select --install
   ```

### Build Commands

```bash
# Build for current architecture
make build

# Build universal binary (both Intel and Apple Silicon)
make universal

# Run tests
make test

# Clean build artifacts
make clean
```

### Manual Build

If you prefer not to use Make:

```bash
# Build Rust library
cd spacesaver-core
cargo build --release

# The build script will generate the C header in include/spacesaver.h

# Build Swift wrapper and create bundle
cd ..
./scripts/build.sh
```

## Configuration

Click "Screen Saver Options..." in System Preferences to configure:

- **NASA API Key**: By default, uses the demo key which has rate limits. Get a free API key at [api.nasa.gov](https://api.nasa.gov) for higher limits.
- **Cache Management**: View cached image count and clear the cache if needed.
- **Fetch Images**: Manually trigger fetching of new images.

### Configuration File

Settings are stored in:
```
~/Library/Application Support/com.spacesaver.Spacesaver/config.json
```

Example configuration:
```json
{
  "api_key": "DEMO_KEY",
  "cache_size": 50,
  "transition_interval": 30.0,
  "show_title": true,
  "show_description": false,
  "transition_duration": 2.0,
  "max_history_days": 365,
  "prefetch_enabled": true
}
```

### Image Cache

Cached images are stored in:
```
~/Library/Caches/com.spacesaver.Spacesaver/
```

## Project Structure

```
Spacesaver/
├── spacesaver-core/          # Rust core library
│   ├── src/
│   │   ├── lib.rs           # Library entry point
│   │   ├── api.rs           # NASA APOD API client
│   │   ├── cache.rs         # Image cache management
│   │   ├── config.rs        # Configuration handling
│   │   ├── error.rs         # Error types
│   │   ├── ffi.rs           # C FFI for Swift integration
│   │   └── image_manager.rs # Image fetching and management
│   ├── Cargo.toml
│   ├── build.rs             # Generates C header
│   └── cbindgen.toml        # Header generation config
├── swift-wrapper/
│   └── Spacesaver/
│       ├── SpacesaverView.swift          # Main screen saver view
│       ├── ConfigureSheetController.swift # Settings panel
│       ├── Spacesaver-Bridging-Header.h  # Swift/C bridge
│       └── Info.plist                    # Bundle metadata
├── scripts/
│   ├── build.sh             # Build script
│   └── install.sh           # Installation script
├── Makefile                 # Build automation
└── README.md
```

## How It Works

1. **Rust Core**: The `spacesaver-core` library handles all NASA API communication, image downloading, and cache management. It exposes a C-compatible FFI.

2. **Swift Wrapper**: The Swift code implements `ScreenSaverView`, calling into the Rust library via FFI to manage images.

3. **Image Flow**:
   - On startup, checks for cached images
   - If cache is empty, fetches random APOD images
   - Background prefetching keeps the cache populated
   - Images are displayed with smooth crossfade transitions

## API Information

This screen saver uses NASA's [APOD API](https://api.nasa.gov/), which is free to use. The default `DEMO_KEY` has rate limits (30 requests/hour, 50 requests/day).

For better performance:
1. Visit [api.nasa.gov](https://api.nasa.gov)
2. Sign up for a free API key
3. Enter the key in Screen Saver Options

## Troubleshooting

### Screen saver doesn't appear in preferences
- Ensure the bundle is in `~/Library/Screen Savers/` or `/Library/Screen Savers/`
- Try logging out and back in

### Images not loading
- Check your internet connection
- Verify the API key is valid
- Check the cache directory for any corrupted files

### Build errors
- Ensure Rust and Xcode CLT are installed
- Run `rustup update` to update Rust
- Check that the correct macOS SDK is installed

### View logs
```bash
log show --predicate 'subsystem == "com.spacesaver.Spacesaver"' --last 1h
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

- NASA for the amazing [APOD](https://apod.nasa.gov/) images and API
- Built with [Rust](https://www.rust-lang.org/) and Swift

## Acknowledgments

All images displayed by this screen saver are from NASA's Astronomy Picture of the Day. Images without copyright information are generally in the public domain. See the [APOD website](https://apod.nasa.gov/apod/lib/about_apod.html) for more information about image permissions.
