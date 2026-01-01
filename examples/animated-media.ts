import sharp from 'sharp'
import { OverlayManager } from './utils.js'
import { join } from 'path'

async function playAnimatedMedia(filePath: string) {
  try {
    console.log(`🎬 Loading animated media: ${filePath}`)

    // 1. Get metadata to know number of frames and dimensions
    const image = sharp(filePath, { animated: true })
    const metadata = await image.metadata()

    if (!metadata.pages || metadata.pages <= 1) {
      console.warn('⚠️ This file is not animated or has only one frame.')
    }

    const width = metadata.width || 400
    const height = metadata.pageHeight || metadata.height || 400
    const frameCount = metadata.pages || 1

    console.log(`✅ Loaded: ${width}x${height} with ${frameCount} frames`)

    // 2. Setup the Overlay Manager
    const manager = new OverlayManager('Animated Overlay (GIF/WebP)', width + 50, height + 50, {
      renderWhenOccluded: true,
    })

    manager.win.setWindowLevel(0)
    manager.win.setRenderWhenOccluded(true)
    console.log('⏳ Pre-decoding frames for performance...')
    const frames: Buffer[] = []
    for (let i = 0; i < frameCount; i++) {
      const frameBuffer = await sharp(filePath, { page: i }).ensureAlpha().raw().toBuffer()
      frames.push(frameBuffer)
      if (i % 10 === 0) process.stdout.write('.')
    }
    console.log('\n🚀 Starting playback...')

    let currentFrame = 0
    const delay = metadata.delay?.[0] || 100

    manager.startLoop(() => {
      manager.drawFrame(25, 25, {
        data: frames[currentFrame],
        width,
        height,
      })

      currentFrame = (currentFrame + 1) % frameCount
    }, 1000 / delay) // Set FPS based on frame delay
  } catch (err) {
    console.error('❌ Error playing media:', err)
  }
}

// Usage: bun examples/animated-media.ts path/to/your.gif
const filePath = process.argv[2]
if (!filePath) {
  console.log('❌ Please provide a path to a GIF or Animated WebP file.')
  console.log('Example: bun examples/animated-media.ts ./my-animation.webp')
  process.exit(1)
}

playAnimatedMedia(join(process.cwd(), filePath))
