# overlay-napi

A high-performance Node.js native addon for creating transparent overlay windows with GPU-accelerated rendering using Rust, winit, and pixels.

![https://github.com/napi-rs/package-template/actions](https://github.com/napi-rs/package-template/workflows/CI/badge.svg)

## Features

- **Transparent Overlay Windows**: Create always-on-top transparent windows
- **GPU-Accelerated Rendering**: Hardware-accelerated pixel buffer rendering
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Thread-Safe**: Safe concurrent access to overlay state
- **N-API Compatible**: Works with Node.js N-API for ABI stability
- **Click-Through Support**: Windows can be configured for click-through functionality

## Installation

```bash
npm install overlay-napi
```

## Usage

### Basic Example

```javascript
const { Overlay } = require('overlay-napi');

// Create a new overlay instance
const overlay = new Overlay();

// Start the overlay window (this blocks the current thread)
overlay.start();

// Update the overlay with pixel data
const width = 800;
const height = 600;
const buffer = new Uint8Array(width * height * 4); // RGBA format

// Fill with some color (red in this example)
for (let i = 0; i < buffer.length; i += 4) {
    buffer[i] = 255;     // R
    buffer[i + 1] = 0;   // G
    buffer[i + 2] = 0;   // B
    buffer[i + 3] = 128; // A (semi-transparent)
}

// Update the overlay frame
overlay.updateFrame(buffer);
```

### API Reference

#### `new Overlay()`
Creates a new overlay instance.

#### `start()`
Starts the overlay window and event loop. **Note**: This method blocks the current thread as it runs the window event loop.

#### `updateFrame(buffer: Buffer)`
Updates the overlay with new pixel data.

- `buffer`: A Node.js Buffer containing RGBA pixel data
- The buffer size must match the overlay frame size (width × height × 4 bytes)

#### `getFrameSize()`
Returns the current frame size as `[width, height]`.

#### `plus_100(input: number): number`
A utility function that adds 100 to the input number (for testing purposes).

## Development

### Prerequisites

- Rust (latest stable version)
- Node.js 16+ with N-API support
- npm or yarn

### Building from Source

```bash
# Install dependencies
npm install

# Build the native addon
npm run build:debug

# Run tests
npm test

# Run Rust unit tests
cargo test
```

### Project Structure

```
overlay-napi/
├── src/
│   └── lib.rs          # Main Rust implementation
├── Cargo.toml          # Rust dependencies
├── package.json        # Node.js package configuration
├── index.d.ts          # TypeScript definitions
├── index.js            # JavaScript entry point
└── README.md           # This file
```

## Technical Details

### Architecture

The project uses a multi-layered architecture:

1. **Rust Core**: Implements the overlay window using winit for window management and pixels for GPU-accelerated rendering
2. **N-API Bridge**: Provides Node.js bindings using napi-rs
3. **JavaScript API**: Exposes a clean JavaScript interface

### Dependencies

- **winit**: Cross-platform window creation and event handling
- **pixels**: Hardware-accelerated pixel buffer rendering
- **napi**: Node.js N-API bindings for Rust
- **raw-window-handle**: Cross-platform raw window handle abstraction

### Performance Considerations

- The overlay runs in a separate thread to avoid blocking the main Node.js event loop
- GPU-accelerated rendering provides smooth performance for real-time updates
- Thread-safe state management ensures safe concurrent access

## Testing

The project includes comprehensive tests:

### Rust Unit Tests
```bash
cargo test
```

### JavaScript Integration Tests
```bash
npm test
```

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows  | ✅ Full | Transparent overlays and click-through supported |
| macOS    | ✅ Full | Transparent overlays supported |
| Linux    | ✅ Full | Requires compositor for transparency |

## Troubleshooting

### Common Issues

1. **Build Failures**: Ensure you have the latest Rust toolchain installed
2. **Window Not Appearing**: Check that the overlay is started in the main thread
3. **Performance Issues**: Use appropriate buffer sizes and update frequencies

### Debug Build

Use the debug build for development and troubleshooting:

```bash
npm run build:debug
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [napi-rs](https://github.com/napi-rs/napi-rs) for Node.js native addon development
- Uses [winit](https://github.com/rust-windowing/winit) for cross-platform window management
- Rendering powered by [pixels](https://github.com/parasyte/pixels) crate
