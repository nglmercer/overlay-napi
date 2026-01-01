import { test, expect } from 'bun:test'

test('native binding loads correctly', () => {
  try {
    // Try to import the native binding
    const binding = require('../index.js')

    // Check if we have the expected exports
    expect(binding).toBeDefined()
    expect(typeof binding.OverlayApp).toBe('function')
    expect(typeof binding.OverlayWindow).toBe('function')
    expect(typeof binding.createColor).toBe('function')
    expect(typeof binding.createRgbaBuffer).toBe('function')
    expect(typeof binding.drawPixel).toBe('function')

    console.log('Native binding loaded successfully')
    console.log('Available exports:', Object.keys(binding))
  } catch (error) {
    console.error('Failed to load native binding:', error.message)
    console.error('Error cause:', error.cause)
    throw error
  }
})

test('basic color creation works', () => {
  try {
    const binding = require('../index.js')

    // Test basic color creation
    const redColor = binding.createColor(255, 0, 0, 255)
    expect(redColor).toBeDefined()

    const blueColor = binding.colorBlue
    expect(blueColor).toBeDefined()

    console.log('Color creation test passed')
  } catch (error) {
    console.error('Color creation test failed:', error.message)
    throw error
  }
})
