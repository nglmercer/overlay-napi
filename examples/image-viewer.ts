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
  loadImage,
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
let width = 0 // Default to 0 to use image size
let height = 0
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

// Real image loader - uses native Rust implementation
function loadImageData(imagePath: string): { data: Buffer; width: number; height: number } | null {
  try {
    console.log('📖 Decoding image natively...')
    const result = loadImage(imagePath)
    return result
  } catch (error) {
    console.error(`❌ Error decoding image: ${error}`)
    return null
  }
}

// Main application
async function main() {
  console.log('🚀 Starting Overlay Image Viewer...')

  // Load image data
  /*   const imageData = loadImageData(imagePath)
  if (!imageData) {
    console.error('❌ Failed to load image')
    process.exit(1)
  } */

  //console.log(`imageData`,imageData)

  // Create overlay
  console.log('🔧 Creating overlay...')
  const overlay = new Overlay()

  try {
    console.log('⚙️  Configuring overlay before initialization...')

    // Configure overlay BEFORE starting (this stores the config for initial creation)
    overlay.setSize(width || 800, height || 600)
    overlay.setPosition(x, y)
    overlay.setTitle(title)

    console.log('📋 Configuration:')
    console.log(`   Position: (${x}, ${y})`)
    console.log(`   Size: ${width || 800}x${height || 600}`)
    console.log(`   Title: ${title}`)

    console.log('🎨 Setting initial frame data...')

    // Set the initial frame data BEFORE starting
    // Using dummy data for now since we're not loading an image
    /*     const dummyData = Buffer.alloc(800 * 600 * 4, 0xFF); // White pixels
    overlay.updateFrame(dummyData); */

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
    console.error(`error:`, error)

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
