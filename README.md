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

### Advanced Example with Window Controls

```javascript
const { Overlay, WindowLevel, createColor, color_red, color_blue } = require('overlay-napi');

// Create a new overlay instance
const overlay = new Overlay();

// Start the overlay window
overlay.start();

// Set window properties
overlay.setTitle('My Overlay');
overlay.setWindowLevel(WindowLevel.AlwaysOnTop);
overlay.setPosition(100, 100);
overlay.setSize(400, 300);

// Show the window
overlay.show();

// Clear the frame with a blue background
overlay.clearFrame(color_blue());

// Draw a red rectangle
overlay.drawRectangle(50, 50, 100, 100, color_red());

// Create a custom color
const customColor = createColor(255, 128, 0, 255); // Orange
overlay.drawRectangle(200, 100, 80, 60, customColor);

// Get window information
const position = overlay.getPosition();
const size = overlay.getSize();
console.log(`Window at (${position.x}, ${position.y}), size: ${size.width}x${size.height}`);

// Check if window is visible
if (overlay.isVisible()) {
    console.log('Window is visible');
}

// Request a redraw
overlay.requestRedraw();
```

### API Reference

#### Core Methods

##### `new Overlay()`
Creates a new overlay instance.

##### `start()`
Starts the overlay window and event loop. **Note**: This method blocks the current thread as it runs the window event loop.

##### `updateFrame(buffer: Buffer)`
Updates the overlay with new pixel data.

- `buffer`: A Node.js Buffer containing RGBA pixel data
- The buffer size must match the overlay frame size (width × height × 4 bytes)

##### `getFrameSize()`
Returns the current frame size as `[width, height]`.

#### Window Control Methods

##### `show()`
Shows the overlay window.

##### `hide()`
Hides the overlay window.

##### `setPosition(x: number, y: number)`
Sets the window position on screen.

##### `getPosition()`
Returns the current window position as `{x, y}`.

##### `setSize(width: number, height: number)`
Sets the window size.

##### `getSize()`
Returns the current window size as `{width, height}`.

##### `setTitle(title: string)`
Sets the window title.

##### `setWindowLevel(level: WindowLevel)`
Sets the window level (Normal, AlwaysOnTop, AlwaysOnBottom).

##### `requestRedraw()`
Requests a window redraw.

##### `isVisible()`
Returns whether the window is currently visible.

#### Rendering Methods

##### `clearFrame(color: Color)`
Clears the entire frame with a solid color.

##### `drawRectangle(x: number, y: number, width: number, height: number, color: Color)`
Draws a filled rectangle at the specified position.

#### Utility Functions

##### `createColor(r: number, g: number, b: number, a: number): Color`
Creates a new Color object.

##### `createPosition(x: number, y: number): WindowPosition`
Creates a new WindowPosition object.

##### `createSize(width: number, height: number): WindowSize`
Creates a new WindowSize object.

##### Predefined Colors
- `color_red()`: Returns red color (255, 0, 0, 255)
- `color_green()`: Returns green color (0, 255, 0, 255)
- `color_blue()`: Returns blue color (0, 0, 255, 255)
- `color_black()`: Returns black color (0, 0, 0, 255)
- `color_white()`: Returns white color (255, 255, 255, 255)
- `color_transparent()`: Returns transparent color (0, 0, 0, 0)

### Types and Enums

#### `WindowLevel` Enum
- `WindowLevel.Normal`: Normal window level
- `WindowLevel.AlwaysOnTop`: Window stays on top of other windows
- `WindowLevel.AlwaysOnBottom`: Window stays behind other windows

#### `Color` Object
```javascript
{
    r: number, // Red component (0-255)
    g: number, // Green component (0-255)
    b: number, // Blue component (0-255)
    a: number  // Alpha component (0-255)
}
```

#### `WindowPosition` Object
```javascript
{
    x: number, // X coordinate
    y: number  // Y coordinate
}
```

#### `WindowSize` Object
```javascript
{
    width: number,  // Width in pixels
    height: number  // Height in pixels
}
```

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
