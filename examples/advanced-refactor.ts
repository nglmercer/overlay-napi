import { OverlayApp, createWindowConfig, OverlayEvent, createColor } from '../index.js'

async function main() {
  console.log('🚀 Starting Advanced Overlay Refactor Demo...')

  try {
    const app = new OverlayApp()
    const config = createWindowConfig()
    config.title = 'Advanced Overlay'
    config.width = 1000
    config.height = 700
    config.transparent = true
    config.decorations = true // Let's show decorations for this demo
    config.resizable = true
    config.alwaysOnTop = true
    console.log('🏗️  Creating window...')
    const win = app.createWindow(config)

    // NEW: Register event listener
    win.onEvent((event: any) => {
      console.log(`\n🔔 Event Received: ${event}`)

      if (event === OverlayEvent.CloseRequested) {
        console.log('👋 Window closing...')
        process.exit(0)
      }
    })

    const bgColor = createColor(20, 20, 25, 180)
    const rectColor = createColor(0, 255, 150, 255)

    console.log('🎮 Controls:')
    console.log('- Use win.minimize(), win.maximize(), win.restore()')
    console.log('- Use win.setFullscreen(true/false)')
    console.log('- Events like Resized, Moved, Focused are logged')

    let x = 0
    let direction = 1

    // Demonstrate some methods
    setTimeout(() => {
      console.log('🛠️  Testing minimize in 2 seconds...')
      setTimeout(() => win.minimize(), 2000)
      setTimeout(() => {
        console.log('🛠️  Restoring window in 2 seconds...')
        win.restore()
      }, 4000)
    }, 1000)

    function tick() {
      try {
        const shouldExit = app.pollEvents()
        if (shouldExit) process.exit(0)

        x += 5 * direction
        if (x > 800 || x < 0) direction *= -1

        win.clearFrame(bgColor)
        win.drawRectangle(x, 250, 200, 200, rectColor)
        win.render()

        setTimeout(tick, 16) // ~60 FPS
      } catch (e) {
        console.error('❌ Error:', e)
        process.exit(1)
      }
    }

    tick()
  } catch (err) {
    console.error('❌ Error:', err)
  }
}

main()
