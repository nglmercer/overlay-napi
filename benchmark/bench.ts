import { Bench } from 'tinybench'
import { Buffer } from 'buffer'

// Import overlay functions - we'll need to create a mock overlay for benchmarking
// since the actual overlay requires a window system
import { 
  createColor, 
  colorRed, 
  colorBlue, 
  colorGreen,
  createPosition,
  createSize 
} from '../index.js'

// Mock frame buffer operations for benchmarking
function createFrameBuffer(width: number, height: number): Buffer {
  const size = width * height * 4 // RGBA
  return Buffer.alloc(size)
}

function fillBufferJavaScript(buffer: Buffer, color: [number, number, number, number]): void {
  for (let i = 0; i < buffer.length; i += 4) {
    buffer[i] = color[0]     // R
    buffer[i + 1] = color[1] // G
    buffer[i + 2] = color[2] // B
    buffer[i + 3] = color[3] // A
  }
}

function drawRectangleJavaScript(
  buffer: Buffer, 
  x: number, 
  y: number, 
  width: number, 
  height: number, 
  color: [number, number, number, number],
  frameWidth: number
): void {
  for (let dy = 0; dy < height; dy++) {
    for (let dx = 0; dx < width; dx++) {
      const px = x + dx
      const py = y + dy
      const index = (py * frameWidth + px) * 4
      
      if (index + 3 < buffer.length) {
        buffer[index] = color[0]     // R
        buffer[index + 1] = color[1] // G
        buffer[index + 2] = color[2] // B
        buffer[index + 3] = color[3] // A
      }
    }
  }
}

// Benchmark suite
const b = new Bench({ time: 100 }) // Run for 100ms per benchmark

const FRAME_WIDTH = 800
const FRAME_HEIGHT = 600
const FRAME_SIZE = FRAME_WIDTH * FRAME_HEIGHT * 4

// Color operations benchmark
b.add('Native createColor', () => {
  createColor(255, 128, 64, 255)
})

b.add('JavaScript color object creation', () => {
  return { r: 255, g: 128, b: 64, a: 255 }
})

// Predefined colors benchmark
b.add('Native predefined color access', () => {
  colorRed()
  colorGreen()
  colorBlue()
})

b.add('JavaScript predefined colors', () => {
  const red = { r: 255, g: 0, b: 0, a: 255 }
  const green = { r: 0, g: 255, b: 0, a: 255 }
  const blue = { r: 0, g: 0, b: 255, a: 255 }
  return { red, green, blue }
})

// Position and size operations benchmark
b.add('Native createPosition', () => {
  createPosition(100, 200)
})

b.add('JavaScript position object creation', () => {
  return { x: 100, y: 200 }
})

b.add('Native createSize', () => {
  createSize(800, 600)
})

b.add('JavaScript size object creation', () => {
  return { width: 800, height: 600 }
})

// Frame buffer operations benchmark
const buffer = createFrameBuffer(FRAME_WIDTH, FRAME_HEIGHT)
const redColor: [number, number, number, number] = [255, 0, 0, 255]
const blueColor: [number, number, number, number] = [0, 0, 255, 255]

b.add('JavaScript buffer fill (800x600)', () => {
  fillBufferJavaScript(buffer, redColor)
})

b.add('JavaScript rectangle draw (100x100)', () => {
  drawRectangleJavaScript(buffer, 100, 100, 100, 100, blueColor, FRAME_WIDTH)
})

// Memory allocation benchmark
b.add('Native Buffer allocation (800x600 RGBA)', () => {
  Buffer.alloc(FRAME_SIZE)
})

b.add('JavaScript Uint8Array allocation (800x600 RGBA)', () => {
  new Uint8Array(FRAME_SIZE)
})

// Color manipulation benchmark
b.add('Native color to RGBA conversion', () => {
  const color = createColor(128, 64, 192, 255)
  // Simulate RGBA conversion (would be internal in native code)
  return [color.r, color.g, color.b, color.a]
})

b.add('JavaScript color array creation', () => {
  return [128, 64, 192, 255]
})

console.log('Running overlay-napi benchmarks...\n')

await b.run()

console.table(b.table())

// Additional performance metrics
console.log('\n--- Performance Analysis ---')
console.log(`Frame size: ${FRAME_WIDTH}x${FRAME_HEIGHT} (${FRAME_SIZE} bytes)`)
console.log(`Total pixels: ${FRAME_WIDTH * FRAME_HEIGHT}`)

const fastest = b.tasks.reduce((prev: any, current: any) => {
  const prevMean = prev.result && 'mean' in prev.result ? prev.result.mean : Infinity
  const currentMean = current.result && 'mean' in current.result ? current.result.mean : Infinity
  return prevMean < currentMean ? prev : current
})

const slowest = b.tasks.reduce((prev: any, current: any) => {
  const prevMean = prev.result && 'mean' in prev.result ? prev.result.mean : 0
  const currentMean = current.result && 'mean' in current.result ? current.result.mean : 0
  return prevMean > currentMean ? prev : current
})

if (fastest.result && slowest.result) {
  console.log(`\nFastest operation: ${fastest.name}`)
  console.log(`Slowest operation: ${slowest.name}`)
  const slowestMean = (slowest.result as any).mean || 0
  const fastestMean = (fastest.result as any).mean || 1
  console.log(`Performance ratio: ${(slowestMean / fastestMean).toFixed(2)}x`)
}

// Memory usage estimation
const memUsage = process.memoryUsage()
console.log('\n--- Memory Usage ---')
console.log(`RSS: ${(memUsage.rss / 1024 / 1024).toFixed(2)} MB`)
console.log(`Heap Used: ${(memUsage.heapUsed / 1024 / 1024).toFixed(2)} MB`)
console.log(`Heap Total: ${(memUsage.heapTotal / 1024 / 1024).toFixed(2)} MB`)
