import { OverlayApp, createColor, createWindowConfig } from '../index.js'

async function main() {
  console.log('🚀 Starting Modern Overlay API with Node.js Loop...')

  try {
    const app = new OverlayApp()
    const config = createWindowConfig()
    config.title = 'Manual Node Loop'
    config.width = 800
    config.height = 600
    config.transparent = true

    console.log('🏗️  Creating window...')
    const win = app.createWindow(config)

    const bgColor = createColor(30, 30, 30, 200)
    const rectColor = createColor(0, 200, 255, 255)

    let x = 0
    let direction = 1
    let frameCount = 0

    console.log('🔄 Loop starting...')

    function tick() {
      try {
        // console.log('Tick start');
        const shouldExit = app.pollEvents()
        if (shouldExit) {
          console.log('\n👋 Window close requested, exiting...')
          process.exit(0)
        }

        frameCount++
        x += 2 * direction
        if (x > 600 || x < 0) direction *= -1

        win.clearFrame(bgColor)
        win.drawRectangle(x, 200, 150, 150, rectColor)
        win.render()

        if (frameCount % 10 === 0) {
          process.stdout.write('.')
        }

        setTimeout(tick, 1)
      } catch (e) {
        console.error('\n❌ Error in tick:', e)
        process.exit(1)
      }
    }

    tick()
  } catch (err) {
    console.error('❌ Initialization error:', err)
    process.exit(1)
  }
}

main()
