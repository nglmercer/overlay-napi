import { test, expect } from 'bun:test'
import * as path from 'path'

// Helper function to safely load binding with retry logic
function loadBindingWithRetry(maxRetries = 3) {
  let lastError = null

  for (let i = 0; i < maxRetries; i++) {
    try {
      // Try different possible paths for the binding
      const possiblePaths = [
        '../index.js',
        './index.js',
        path.join(__dirname, '..', 'index.js'),
        path.join(__dirname, 'index.js'),
      ]

      for (const tryPath of possiblePaths) {
        try {
          return require(tryPath)
        } catch (e) {
          // Continue to next path
        }
      }

      // If all paths fail, try with explicit native library path
      if (process.env.NAPI_RS_NATIVE_LIBRARY_PATH) {
        process.env.NAPI_RS_FORCE_WASI = '0'
        return require('../index.js')
      }

      throw new Error('Could not load binding from any path')
    } catch (error) {
      lastError = error
      console.log(`Attempt ${i + 1} failed: ${(error as Error).message}`)
      if (i < maxRetries - 1) {
        // Wait a bit before retrying
        setTimeout(() => {}, 100)
      }
    }
  }

  throw lastError
}

test('native binding loads correctly', () => {
  try {
    console.log('=== Starting native binding test ===')
    console.log('Platform:', process.platform)
    console.log('Architecture:', process.arch)
    console.log('Node version:', process.version)
    console.log('Current directory:', process.cwd())

    // List files in current directory for debugging
    const fs = require('fs')
    const files = fs.readdirSync('.')
    const nodeFiles = files.filter((f: string) => f.endsWith('.node'))
    console.log('Files in directory:', files.length)
    console.log('.node files:', nodeFiles)

    // Try to load the binding
    const binding = loadBindingWithRetry()

    // Check if we have the expected exports
    expect(binding).toBeDefined()
    expect(typeof binding.OverlayApp).toBe('function')
    expect(typeof binding.OverlayWindow).toBe('function')
    expect(typeof binding.createColor).toBe('function')
    expect(typeof binding.createRgbaBuffer).toBe('function')
    expect(typeof binding.drawPixel).toBe('function')

    console.log('✓ Native binding loaded successfully')
    console.log('✓ Available exports count:', Object.keys(binding).length)
    console.log('✓ First 5 exports:', Object.keys(binding).slice(0, 5).join(', '))
  } catch (error) {
    console.error('✗ Failed to load native binding:', (error as Error).message)
    if ((error as any).cause) {
      console.error('✗ Error cause:', (error as any).cause.message)
    }
    console.error('✗ This might be due to:')
    console.error('  - Missing .node file for this platform')
    console.error('  - Incorrect file naming')
    console.error('  - Platform/architecture mismatch')
    console.error('  - Missing optional dependencies')

    // Provide more context about the error
    if ((error as any).code === 'MODULE_NOT_FOUND') {
      console.error('✗ Module not found - checking possible solutions...')
    }

    throw error
  }
})

test('basic color creation works', () => {
  try {
    console.log('=== Starting color creation test ===')

    const binding = loadBindingWithRetry()

    // Test basic color creation
    console.log('Testing createColor function...')
    const redColor = binding.createColor(255, 0, 0, 255)
    expect(redColor).toBeDefined()
    console.log('✓ createColor(255, 0, 0, 255) works')

    console.log('Testing predefined colors...')
    const blueColor = binding.colorBlue
    expect(blueColor).toBeDefined()
    console.log('✓ colorBlue is defined')

    const redColorPre = binding.colorRed
    expect(redColorPre).toBeDefined()
    console.log('✓ colorRed is defined')

    console.log('✓ Color creation test passed')
  } catch (error) {
    console.error('✗ Color creation test failed:', (error as Error).message)
    throw error
  }
})

test('CI environment verification', () => {
  try {
    console.log('=== Starting CI environment verification ===')

    // Check if we're in a CI environment
    const isCI = process.env.CI === 'true' || process.env.GITHUB_ACTIONS === 'true'
    console.log('CI Environment:', isCI ? 'Yes' : 'No')

    // Verify critical files exist
    const fs = require('fs')
    const criticalFiles = ['index.js', 'index.d.ts']

    criticalFiles.forEach((file) => {
      if (fs.existsSync(file)) {
        console.log(`✓ ${file} exists`)
      } else {
        throw new Error(`Critical file missing: ${file}`)
      }
    })

    // Check for .node files
    const nodeFiles = fs.readdirSync('.').filter((f: string) => f.endsWith('.node'))
    if (nodeFiles.length === 0) {
      console.warn('⚠ No .node files found in root directory')
    } else {
      console.log(`✓ Found ${nodeFiles.length} .node files`)
    }

    console.log('✓ CI environment verification passed')
  } catch (error) {
    console.error('✗ CI environment verification failed:', (error as Error).message)
    throw error
  }
})
