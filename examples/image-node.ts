import { OverlayApp, createWindowConfig, createColor } from '../index.js'
import sharp from 'sharp'
import { join } from 'path'

const basePath = process.cwd()
async function main() {
  console.log('🚀 Starting Image Overlay with Sharp (Node.js Decoding)...')

  try {
    const imagePath = join(basePath, 'examples', 'SAVEDQR.png')
    console.log(`📸 Loading image: ${imagePath}`)

    // Use sharp to decode the image to raw RGBA8 buffer
    const { data, info } = await sharp(imagePath)
      .ensureAlpha() // Ensure we have 4 channels (RGBA)
      .raw()
      .toBuffer({ resolveWithObject: true })

    console.log(`✅ Image decoded: ${info.width}x${info.height}, channels: ${info.channels}`)

    const app = new OverlayApp()
    const config = createWindowConfig()
    config.title = 'Sharp Image Overlay'
    config.width = info.width + 100
    config.height = info.height + 100
    config.transparent = true
    config.decorations = true // Showing decorations for better visibility

    const win = app.createWindow(config)
    const bgColor = createColor(40, 44, 52, 200)

    console.log('🔄 Starting render loop...')

    function tick() {
      try {
        const shouldExit = app.pollEvents()
        if (shouldExit) process.exit(0)

        // Clear with semi-transparent background
        win.clearFrame(bgColor)

        // Draw the image using the raw buffer from sharp
        win.drawImage(50, 50, {
          data: data,
          width: info.width,
          height: info.height,
        })

        win.render()
        setTimeout(tick, 16)
      } catch (e) {
        console.error('❌ Error during tick:', e)
        process.exit(1)
      }
    }

    tick()
  } catch (err) {
    console.error('❌ Error:', err)
  }
}

main()
