const fs = require('fs')
const path = require('path')

console.log('=== CI Verification Script ===')

// Verificar archivos críticos
const criticalFiles = ['index.js', 'index.d.ts', 'scripts/post-build.js', 'scripts/pre-test.js']

console.log('\n1. Checking critical files:')
criticalFiles.forEach((file) => {
  if (fs.existsSync(file)) {
    console.log(`  ✓ ${file} exists`)
  } else {
    console.log(`  ✗ ${file} missing`)
    process.exit(1)
  }
})

// Verificar archivos .node
console.log('\n2. Checking native binding files:')
const nodeFiles = fs.readdirSync('.').filter((file) => file.endsWith('.node'))
if (nodeFiles.length > 0) {
  console.log(`  Found ${nodeFiles.length} .node files:`)
  nodeFiles.forEach((file) => {
    const stats = fs.statSync(file)
    console.log(`    - ${file} (${stats.size} bytes)`)
  })
} else {
  console.log('  ⚠ No .node files found in root directory')
}

// Verificar paquetes opcionales
console.log('\n3. Checking optional packages:')
const optionalPackages = fs
  .readdirSync('.')
  .filter((dir) => dir.startsWith('overlay-napi-') && fs.statSync(dir).isDirectory())

if (optionalPackages.length > 0) {
  console.log(`  Found ${optionalPackages.length} optional packages:`)
  optionalPackages.forEach((pkg) => {
    const packageJsonPath = path.join(pkg, 'package.json')
    const nodeFilePath = path.join(pkg, 'index.node')

    if (fs.existsSync(packageJsonPath) && fs.existsSync(nodeFilePath)) {
      console.log(`    ✓ ${pkg}`)
    } else {
      console.log(`    ✗ ${pkg} (incomplete)`)
    }
  })
} else {
  console.log('  ⚠ No optional packages found')
}

// Verificar plataforma
console.log('\n4. Platform information:')
console.log(`  Platform: ${process.platform}`)
console.log(`  Architecture: ${process.arch}`)
console.log(`  Node version: ${process.version}`)

// Intentar cargar el binding
console.log('\n5. Testing binding load:')
try {
  const binding = require(path.join(__dirname, '..', 'index.js'))
  const exports = Object.keys(binding)
  console.log(`  ✓ Binding loaded successfully`)
  console.log(`  ✓ Found ${exports.length} exports`)
  console.log(`  ✓ First 10 exports: ${exports.slice(0, 10).join(', ')}`)

  // Probar una función simple
  if (binding.createColor) {
    const color = binding.createColor(255, 0, 0, 255)
    console.log(`  ✓ createColor function works`)
  }

  console.log('\n=== Verification completed successfully ===')
  process.exit(0)
} catch (error) {
  console.log(`  ✗ Failed to load binding: ${error.message}`)
  if (error.cause) {
    console.log(`    Cause: ${error.cause.message}`)
  }
  console.log('\n=== Verification failed ===')
  process.exit(1)
}
