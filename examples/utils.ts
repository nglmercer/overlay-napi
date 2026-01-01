import { OverlayApp, createWindowConfig, createColor, Color, OverlayWindow } from '../index.js'

export interface FrameData {
  data: Buffer | Uint8Array
  width: number
  height: number
}

/**
 * Shared utility to manage Overlay Application lifecycle and rendering
 */
export class OverlayManager {
  public app: OverlayApp
  public win: OverlayWindow
  public bgColor: Color

  constructor(title: string, width: number, height: number, options: { renderWhenOccluded?: boolean } = {}) {
    this.app = new OverlayApp()
    const config = createWindowConfig()
    config.title = title
    config.width = width
    config.height = height
    config.transparent = true
    config.decorations = true
    config.alwaysOnTop = true
    config.renderWhenOccluded = options.renderWhenOccluded ?? true

    this.win = this.app.createWindow(config)
    this.bgColor = createColor(0, 0, 0, 0) // Fully transparent by default

    // Mitigation for "Drag Freeze" and "Minimize Freeze"
    this.win.onEvent((err, _event) => {
      if (err) return
      // When moving or resizing, Windows blocks the main loop.
      // Requesting a redraw here can help keep the buffer valid.
    })
  }

  /**
   * Main loop to poll events and keep the window alive
   */
  public startLoop(onTick: () => void, fps = 60) {
    const tick = () => {
      try {
        const shouldExit = this.app.pollEvents()
        if (shouldExit) process.exit(0)

        // Check if we should skip rendering based on occlusion (if not forced)
        onTick()

        setTimeout(tick, Math.floor(1000 / fps))
      } catch (e) {
        console.error('❌ Error in render loop:', e)
        process.exit(1)
      }
    }
    tick()
  }

  /**
   * Helper to draw a frame efficiently
   */
  public drawFrame(x: number, y: number, frame: FrameData) {
    this.win.clearFrame(this.bgColor)
    this.win.drawImage(x, y, {
      data: Buffer.from(frame.data),
      width: frame.width,
      height: frame.height,
    })
    this.win.render()
  }
}
