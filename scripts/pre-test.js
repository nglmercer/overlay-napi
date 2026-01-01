const fs = require('fs')
const path = require('path')

console.log('Pre-test setup script')

// Verificar que los archivos necesarios existan
const requiredFiles = ['index.js', 'index.d.ts']
const optionalFiles = fs.readdirSync('.').filter((file) => file.endsWith('.node'))

console.log('Required files check:')
requiredFiles.forEach((file) => {
  if (fs.existsSync(file)) {
    console.log(`  ✓ ${file} exists`)
  } else {
    console.log(`  ✗ ${file} missing`)
  }
})

console.log('Optional .node files:')
if (optionalFiles.length > 0) {
  optionalFiles.forEach((file) => {
    const stats = fs.statSync(file)
    console.log(`  - ${file} (${stats.size} bytes)`)
  })
} else {
  console.log('  No .node files found')
}

// Configurar variables de entorno para ayudar con la carga de bindings
const target = process.env.TARGET
if (optionalFiles.length > 0) {
  let selectedFile = optionalFiles[0]

  if (target) {
    // Intentar encontrar el archivo que coincida con el target
    const targetMatch = optionalFiles.find((f) => f.includes(target.split('-')[0]) || f.includes(target))
    if (targetMatch) {
      selectedFile = targetMatch
    }
  } else {
    // Intentar encontrar el archivo que coincida con la arquitectura actual
    const archMatch = optionalFiles.find((f) => f.includes(process.arch))
    if (archMatch) {
      selectedFile = archMatch
    }
  }

  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = path.resolve(selectedFile)
  console.log(`Set NAPI_RS_NATIVE_LIBRARY_PATH to: ${process.env.NAPI_RS_NATIVE_LIBRARY_PATH}`)
}

// Verificar la plataforma y arquitectura
console.log('\nPlatform info:')
console.log(`  Platform: ${process.platform}`)
console.log(`  Architecture: ${process.arch}`)
console.log(`  Node version: ${process.version}`)

// Intentar cargar el binding para verificar que funcione
try {
  console.log('\nTesting binding load...')
  const binding = require('../index.js')
  console.log('✓ Binding loaded successfully')
  console.log('Available exports:', Object.keys(binding).slice(0, 10).join(', '))
} catch (error) {
  console.error('✗ Failed to load binding:', error.message)
  if (error.cause) {
    console.error('Cause:', error.cause.message)
  }
  // No fallar el script, solo advertir
}

console.log('\nPre-test setup completed')
