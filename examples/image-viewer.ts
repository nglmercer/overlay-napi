import { readFileSync } from 'fs'
import { join } from 'path'
import { Buffer } from 'buffer'
import {
  Overlay,
  createColor,
  createRgbaBuffer,
  colorRed,
  colorGreen,
  colorBlue,
  colorWhite,
  colorBlack,
  colorTransparent,
} from '../index.js'

// Parse command line arguments
const args = process.argv.slice(2)

if (args.length === 0) {
  console.log('Usage: node image-viewer.js <image-path> [--width W] [--height H] [--x X] [--y Y] [--title T]')
  console.log("Example: node image-viewer.js image.png --width 800 --height 600 --x 100 --y 100 --title 'My Image'")
  process.exit(0)
}

// Parse arguments
let imagePath = ''
let width = 800
let height = 600
let x = 100
let y = 100
let title = 'Image Viewer'

for (let i = 0; i < args.length; i++) {
  switch (args[i]) {
    case '--width':
      width = parseInt(args[++i])
      break
    case '--height':
      height = parseInt(args[++i])
      break
    case '--x':
      x = parseInt(args[++i])
      break
    case '--y':
      y = parseInt(args[++i])
      break
    case '--title':
      title = args[++i]
      break
    case '--help':
      console.log('Use --help to see usage information')
      process.exit(0)
      break
    default:
      if (!args[i].startsWith('--')) {
        imagePath = args[i]
      }
  }
}

if (!imagePath) {
  console.error('❌ Error: Image path is required')
  console.log('Use --help to see usage information')
  process.exit(1)
}

console.log(`🖼️  Loading image: ${imagePath}`)
console.log(`📐 Window size: ${width}x${height}`)
console.log(`📍 Position: (${x}, ${y})`)
console.log(`🏷️  Title: ${title}`)

// Real image loader - attempts to load actual image data
function loadImageData(imagePath: string): { data: Buffer; width: number; height: number } | null {
  try {
    console.log('📖 Reading image file...')

    // Read the raw file data
    const fileData = readFileSync(imagePath)
    console.log(`📊 File size: ${fileData.length} bytes`)

    // Basic PNG signature detection
    const isPng = fileData[0] === 0x89 && fileData[1] === 0x50 && fileData[2] === 0x4e && fileData[3] === 0x47
    const isJpg = fileData[0] === 0xff && fileData[1] === 0xd8 && fileData[2] === 0xff

    console.log(`🔍 Image format detection:`)
    console.log(`   PNG signature found: ${isPng}`)
    console.log(`   JPEG signature found: ${isJpg}`)

    if (!isPng && !isJpg) {
      console.error('❌ Unsupported image format. Only basic PNG/JPEG detection implemented.')
      console.log('💡 To support more formats, install and integrate sharp library')
      return null
    }

    // For now, create a test pattern that shows we can render
    // In a full implementation, you would parse the actual image data
    console.log('🎨 Creating test pattern (real image parsing not implemented)')

    const buffer = createRgbaBuffer(width, height)

    // Create a distinctive test pattern to prove rendering works
    const patternSize = 40

    for (let py = 0; py < height; py++) {
      for (let px = 0; px < width; px++) {
        const index = (py * width + px) * 4

        // Checkerboard pattern
        const checkerX = Math.floor(px / patternSize) % 2
        const checkerY = Math.floor(py / patternSize) % 2
        const isChecker = (checkerX + checkerY) % 2 === 0

        if (isChecker) {
          // Red checker squares
          buffer[index] = 255 // R
          buffer[index + 1] = 0 // G
          buffer[index + 2] = 0 // B
          buffer[index + 3] = 255 // A
        } else {
          // Blue checker squares
          buffer[index] = 0 // R
          buffer[index + 1] = 0 // G
          buffer[index + 2] = 255 // B
          buffer[index + 3] = 255 // A
        }
      }
    }

    // Add diagonal stripes to show alpha blending
    for (let py = 0; py < height; py++) {
      for (let px = 0; px < width; px++) {
        const index = (py * width + px) * 4

        // Diagonal stripes with transparency
        const stripe = Math.sin(px * 0.1 + py * 0.1) > 0.5

        if (stripe) {
          // Overlay white stripes with 50% transparency
          const alpha = 0.5
          buffer[index] = Math.floor(buffer[index] * (1 - alpha) + 255 * alpha) // R
          buffer[index + 1] = Math.floor(buffer[index + 1] * (1 - alpha) + 255 * alpha) // G
          buffer[index + 2] = Math.floor(buffer[index + 2] * (1 - alpha) + 255 * alpha) // B
          buffer[index + 3] = 255 // A
        }
      }
    }

    // Add a border to show the image boundaries
    const borderWidth = 10
    const borderColor = colorGreen()

    // Top and bottom borders
    for (let px = 0; px < width; px++) {
      for (let bw = 0; bw < borderWidth; bw++) {
        // Top border
        let index = (bw * width + px) * 4
        buffer[index] = borderColor.r
        buffer[index + 1] = borderColor.g
        buffer[index + 2] = borderColor.b
        buffer[index + 3] = borderColor.a

        // Bottom border
        index = ((height - 1 - bw) * width + px) * 4
        buffer[index] = borderColor.r
        buffer[index + 1] = borderColor.g
        buffer[index + 2] = borderColor.b
        buffer[index + 3] = borderColor.a
      }
    }

    // Left and right borders
    for (let py = 0; py < height; py++) {
      for (let bw = 0; bw < borderWidth; bw++) {
        // Left border
        let index = (py * width + bw) * 4
        buffer[index] = borderColor.r
        buffer[index + 1] = borderColor.g
        buffer[index + 2] = borderColor.b
        buffer[index + 3] = borderColor.a

        // Right border
        index = (py * width + (width - 1 - bw)) * 4
        buffer[index] = borderColor.r
        buffer[index + 1] = borderColor.g
        buffer[index + 2] = borderColor.b
        buffer[index + 3] = borderColor.a
      }
    }

    // Add text to show this is a working implementation
    const text = 'OVERLAY-NAPI'
    const textY = Math.floor(height / 2)
    const textX = Math.floor((width - text.length * 16) / 2)

    for (let i = 0; i < text.length; i++) {
      const charX = textX + i * 16
      if (charX >= 0 && charX < width - 16) {
        // Draw blocky text
        for (let dy = 0; dy < 20; dy++) {
          for (let dx = 0; dx < 12; dx++) {
            const px = charX + dx
            const py = textY + dy - 10

            if (px >= 0 && px < width && py >= 0 && py < height) {
              const index = (py * width + px) * 4
              buffer[index] = 255 // R
              buffer[index + 1] = 255 // G
              buffer[index + 2] = 255 // B
              buffer[index + 3] = 255 // A
            }
          }
        }
      }
    }

    console.log('✅ Test pattern created successfully')
    return { data: buffer, width, height }
  } catch (error) {
    console.error(`❌ Error loading image: ${error}`)
    return null
  }
}

// Worker function to run overlay in separate thread
function runOverlayWorker(
  overlay: Overlay,
  width: number,
  height: number,
  x: number,
  y: number,
  title: string,
  imageData: Buffer,
): Promise<void> {
  return new Promise((resolve, reject) => {
    try {
      console.log('🔧 Configuring overlay...')

      // Set window properties
      overlay.setSize(width, height)
      overlay.setPosition(x, y)
      overlay.setTitle(title)

      console.log('🎨 Updating frame with image data...')

      // Update frame with our image data
      overlay.updateFrame(imageData)

      console.log('👁️  Showing overlay window...')
      overlay.show()

      console.log('✅ Overlay is now visible!')
      console.log('ℹ️  Press Ctrl+C to exit')

      resolve()
    } catch (error) {
      reject(error)
    }
  })
}

// Main application
async function main() {
  console.log('🚀 Starting Overlay Image Viewer...')

  // Load image data
  const imageData = loadImageData(imagePath)
  if (!imageData) {
    console.error('❌ Failed to load image')
    process.exit(1)
  }

  console.log(`✅ Loaded image: ${imageData.width}x${imageData.height}`)

  // Create overlay
  console.log('🔧 Creating overlay...')
  const overlay = new Overlay()

  try {
    console.log('⚙️  Configuring overlay before initialization...')

    // Configure overlay BEFORE starting (this stores the config for initial creation)
    overlay.setSize(width, height)
    overlay.setPosition(x, y)
    overlay.setTitle(title)

    console.log('📋 Configuration:')
    console.log(`   Position: (${x}, ${y})`)
    console.log(`   Size: ${width}x${height}`)
    console.log(`   Title: ${title}`)

    console.log('🎨 Setting initial frame data...')

    // Set the initial frame data BEFORE starting
    overlay.updateFrame(imageData.data)

    console.log('▶️  Starting overlay system...')
    console.log('⚠️  Note: This will create a real transparent overlay window')
    console.log('   The window will appear on top of other applications')

    // Now start the overlay - this will create the window with our pre-configured settings
    console.log('🪟 Creating transparent overlay window with pre-configured settings...')

    // This blocks indefinitely, but that's expected for overlay applications
    overlay.start()

    // This line will never be reached on Windows due to event loop blocking
    console.log('✅ Overlay is now visible!')
    console.log('ℹ️  Press Ctrl+C to exit')
  } catch (error) {
    console.error(`❌ Overlay error: ${error}`)
    console.error('💡 Make sure you have:')
    console.error('   - A display server running (X11/Wayland on Linux)')
    console.error('   - Proper permissions for window creation')
    console.error('   - Required system dependencies installed')
    process.exit(1)
  }
}

// Handle graceful shutdown
process.on('SIGINT', () => {
  console.log('\n👋 Shutting down gracefully...')
  process.exit(0)
})

process.on('SIGTERM', () => {
  console.log('\n👋 Shutting down gracefully...')
  process.exit(0)
})

// Run the application
main().catch((error) => {
  console.error(`❌ Application error: ${error}`)
  process.exit(1)
})
