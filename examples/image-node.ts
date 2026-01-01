import sharp from 'sharp'
import { OverlayManager } from './utils.js'
import { join } from 'path'

async function main() {
  console.log('🚀 Starting Static Image Overlay using Utils...')

  try {
    const imagePath = join(process.cwd(), 'examples', 'SAVEDQR.png')

    // 1. Decode using sharp
    const { data, info } = await sharp(imagePath).ensureAlpha().raw().toBuffer({ resolveWithObject: true })

    console.log(`✅ Image decoded: ${info.width}x${info.height}`)

    // 2. Use our new manager (Much cleaner!)
    const manager = new OverlayManager('Static Image Demo', info.width + 100, info.height + 100)

    manager.startLoop(() => {
      manager.drawFrame(50, 50, {
        data,
        width: info.width,
        height: info.height,
      })
    })
  } catch (err) {
    console.error('❌ Error:', err)
  }
}

main()
