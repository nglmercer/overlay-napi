const fs = require('fs')
const path = require('path')

const platform = process.platform
const arch = process.arch
const target = process.env.TARGET || `${arch}-${platform}`

console.log(`Post-build script for ${platform}-${arch} (target: ${target})`)

// Mapping de plataformas y arquitecturas a los nombres esperados
const platformMap = {
  win32: 'win32',
  darwin: 'darwin',
  linux: 'linux',
}

const archMap = {
  x64: 'x64',
  ia32: 'ia32',
  arm64: 'arm64',
  arm: 'arm',
}

const libcMap = {
  linux: process.env.LIBC || 'gnu',
}

function getExpectedFilename() {
  const platformName = platformMap[platform]
  const archName = archMap[arch]

  if (!platformName || !archName) {
    console.error(`Unsupported platform/architecture: ${platform}/${arch}`)
    return null
  }

  let filename = `overlay-napi.${platformName}-${archName}`

  if (platform === 'linux') {
    filename += `-${libcMap.linux}`
  }

  filename += '.node'

  return filename
}

function renameNodeFiles() {
  const expectedFilename = getExpectedFilename()
  if (!expectedFilename) return

  console.log(`Expected filename: ${expectedFilename}`)

  // Buscar archivos .node en el directorio actual
  const files = fs.readdirSync('.')
  const nodeFiles = files.filter((file) => file.endsWith('.node'))

  console.log(`Found .node files: ${nodeFiles.join(', ')}`)

  if (nodeFiles.length === 0) {
    console.log('No .node files found')
    return
  }

  // Si ya existe el archivo con el nombre esperado, no hacer nada
  if (nodeFiles.includes(expectedFilename)) {
    console.log(`File ${expectedFilename} already exists`)
    return
  }

  // Intentar renombrar el primer archivo .node encontrado
  const sourceFile = nodeFiles[0]
  console.log(`Renaming ${sourceFile} to ${expectedFilename}`)

  try {
    fs.renameSync(sourceFile, expectedFilename)
    console.log(`Successfully renamed ${sourceFile} to ${expectedFilename}`)
  } catch (error) {
    console.error(`Failed to rename file: ${error.message}`)
  }
}

// También crear enlaces simbólicos para los nombres universales de macOS
if (platform === 'darwin') {
  console.log('Creating universal binary symlink for macOS')
  try {
    const universalName = 'overlay-napi.darwin-universal.node'
    const expectedFilename = getExpectedFilename()

    if (expectedFilename && fs.existsSync(expectedFilename)) {
      if (!fs.existsSync(universalName)) {
        fs.symlinkSync(expectedFilename, universalName)
        console.log(`Created symlink: ${universalName} -> ${expectedFilename}`)
      }
    }
  } catch (error) {
    console.error(`Failed to create universal symlink: ${error.message}`)
  }
}

renameNodeFiles()

// Verificar archivos finales
console.log('\nFinal .node files:')
const finalFiles = fs.readdirSync('.').filter((file) => file.endsWith('.node'))
finalFiles.forEach((file) => {
  const stats = fs.statSync(file)
  console.log(`  ${file} (${stats.size} bytes)`)
})
