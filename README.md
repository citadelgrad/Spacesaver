# Spacesaver

A macOS screen saver featuring stunning space imagery — bundled nebula and galaxy photos, or live images from NASA.

![macOS](https://img.shields.io/badge/macOS-10.15+-blue)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![Swift](https://img.shields.io/badge/Swift-5.5+-red)
![License](https://img.shields.io/badge/License-MIT-green)

## Features

- **Stunning Bundled Images**: Ships with a curated collection of nebula and galaxy photographs — no internet required
- **NASA Image Sources**: Optionally fetch images from NASA's Image Library or Astronomy Picture of the Day API
- **Smart Caching**: Downloads and caches images locally for offline viewing
- **Smooth Transitions**: Elegant crossfade transitions between images
- **Information Overlay**: Displays image title, date, and copyright information
- **Configurable**: Choose your image source and settings through the Screen Saver Options panel
- **Built with Rust**: Core image management written in Rust for performance and reliability
- **Signed with Developer ID**: No Gatekeeper warnings on install

## Requirements

- macOS 10.15 (Catalina) or later
- Rust 1.70+ (for building from source)
- Xcode Command Line Tools (for building from source)

## Installation

### From Release

1. Download the latest `Spacesaver-v*.zip` from the [Releases](https://github.com/citadelgrad/Spacesaver/releases) page
2. Unzip and double-click `Spacesaver.saver` to install
3. Open System Settings > Screen Saver and select "Spacesaver"

### From Source

```bash
git clone https://github.com/citadelgrad/Spacesaver.git
cd Spacesaver
./scripts/build.sh
cp -r build/Spacesaver.saver ~/Library/Screen\ Savers/
```

## Configuration

Click **Options...** in System Settings > Screen Saver to configure:

- **Image Source**: Choose between:
  - **Bundled Images** (default) — curated nebula and galaxy photos, no internet needed
  - **NASA Image Library** — space images from NASA's public library (no API key required)
  - **NASA APOD** — Astronomy Picture of the Day (requires free API key)
  - **APOD with Fallback** — tries APOD first, falls back to NASA Image Library
- **NASA API Key**: Required only for APOD sources. Get a free key at [api.nasa.gov](https://api.nasa.gov).
- **Cache Management**: View cached image count and clear the cache
- **Fetch Images**: Manually trigger fetching of new images (for NASA sources)

### Configuration File

Settings are stored in:
```
~/Library/Application Support/com.spacesaver.Spacesaver/config.json
```

### Image Cache

Cached images are stored in:
```
~/Library/Caches/com.spacesaver.Spacesaver/
```

## Building

### Prerequisites

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install Xcode Command Line Tools:
   ```bash
   xcode-select --install
   ```

### Build

```bash
./scripts/build.sh
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
├── resources/
│   └── bundled-images/       # Bundled nebula/galaxy images
├── scripts/
│   ├── build.sh             # Build script
│   └── release.sh           # Release automation
└── README.md
```

## How It Works

1. **Rust Core**: The `spacesaver-core` library handles NASA API communication, image downloading, and cache management, exposed via a C-compatible FFI.

2. **Swift Wrapper**: Implements `ScreenSaverView`, calling into the Rust library to manage images and transitions.

3. **Image Flow**:
   - On startup, loads bundled images (or checks cache for NASA sources)
   - For NASA sources, background prefetching keeps the cache populated
   - Images are displayed with smooth crossfade transitions

## Troubleshooting

### Screen saver doesn't appear in preferences
- Ensure the bundle is in `~/Library/Screen Savers/` or `/Library/Screen Savers/`
- Try logging out and back in

### Images not loading
- If using NASA sources, check your internet connection
- Switch to "Bundled Images" in Options for offline use

### View logs
```bash
cat ~/Library/Caches/com.spacesaver.Spacesaver/spacesaver.log
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

- NASA for the [APOD](https://apod.nasa.gov/) and [Image Library](https://images.nasa.gov/) APIs
- Built with [Rust](https://www.rust-lang.org/) and Swift
