const fs = require('fs')
const path = require('path')

let currentPlatform = process.platform
let currentArch = process.arch
const target = process.env.TARGET

if (target) {
  console.log(`Using TARGET environment variable: ${target}`)
  if (target.includes('linux')) currentPlatform = 'linux'
  if (target.includes('darwin') || target.includes('apple')) currentPlatform = 'darwin'
  if (target.includes('windows') || target.includes('win32') || target.includes('msvc')) currentPlatform = 'win32'

  if (target.includes('x86_64') || target.includes('x64')) currentArch = 'x64'
  else if (target.includes('aarch64') || target.includes('arm64')) currentArch = 'arm64'
  else if (target.includes('armv7') || target.includes('armhf')) currentArch = 'arm'
  else if (target.includes('i686')) currentArch = 'ia32'
}

const platform = currentPlatform
const arch = currentArch

console.log(`Post-build script for ${platform}-${arch} (target: ${target || 'default'})`)

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

  // Buscar el archivo original (con el nombre que espera index.js)
  let sourceFile = null
  const possibleOriginalNames = [
    `overlay-napi.${platformMap[platform]}-${archMap[arch]}-${platform === 'win32' ? 'msvc' : 'gnu'}.node`,
    `overlay-napi.${platformMap[platform]}-${archMap[arch]}.node`,
  ]

  for (const name of possibleOriginalNames) {
    if (nodeFiles.includes(name)) {
      sourceFile = name
      break
    }
  }

  if (!sourceFile) {
    sourceFile = nodeFiles[0] // Usar el primer archivo como fallback
  }

  console.log(`Using source file: ${sourceFile}`)

  try {
    // Crear el archivo con el nombre esperado por el sistema
    if (!fs.existsSync(expectedFilename)) {
      fs.copyFileSync(sourceFile, expectedFilename)
      console.log(`Successfully created ${expectedFilename}`)
    } else {
      console.log(`File ${expectedFilename} already exists`)
    }

    // Crear copias con los nombres que espera index.js para cada plataforma
    const platformArch = `${platformMap[platform]}-${archMap[arch]}`
    const indexJsNames = [
      `overlay-napi.${platformArch}.node`,
      `overlay-napi.${platformArch}-${platform === 'win32' ? 'msvc' : 'gnu'}.node`,
    ]

    indexJsNames.forEach((name) => {
      if (!fs.existsSync(name) && name !== sourceFile) {
        fs.copyFileSync(sourceFile, name)
        console.log(`Created copy for index.js: ${name}`)
      }
    })
  } catch (error) {
    console.error(`Failed to copy file: ${error.message}`)
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

// Crear enlaces para los nombres de paquetes opcionales esperados
function createOptionalPackageLinks() {
  // Use the calculated platform and arch instead of process.platform/arch

  // Mapeo de nombres de paquetes opcionales
  const packageNameMap = {
    win32: {
      x64: 'overlay-napi-win32-x64-msvc',
      ia32: 'overlay-napi-win32-ia32-msvc',
      arm64: 'overlay-napi-win32-arm64-msvc',
    },
    darwin: {
      x64: 'overlay-napi-darwin-x64',
      arm64: 'overlay-napi-darwin-arm64',
    },
    linux: {
      x64: 'overlay-napi-linux-x64-gnu',
      arm64: 'overlay-napi-linux-arm64-gnu',
      arm: 'overlay-napi-linux-arm-gnueabihf',
    },
  }

  const expectedPackageName = packageNameMap[platform]?.[arch]
  if (!expectedPackageName) {
    console.log(`No optional package name mapping for ${platform}-${arch}`)
    return
  }

  const expectedFilename = getExpectedFilename()
  if (!expectedFilename) return

  if (!fs.existsSync(expectedFilename)) {
    console.log(`Expected file ${expectedFilename} not found, skipping package link creation`)
    return
  }

  // Crear un directorio para el paquete opcional
  const packageDir = expectedPackageName
  if (!fs.existsSync(packageDir)) {
    fs.mkdirSync(packageDir, { recursive: true })
  }

  // Crear package.json para el paquete opcional
  const packageJson = {
    name: expectedPackageName,
    version: '1.0.0',
    main: 'index.node',
    files: ['index.node'],
  }

  fs.writeFileSync(path.join(packageDir, 'package.json'), JSON.stringify(packageJson, null, 2))

  // Copiar el archivo .node como index.node
  const targetFile = path.join(packageDir, 'index.node')
  try {
    fs.copyFileSync(expectedFilename, targetFile)
    console.log(`Created optional package: ${expectedPackageName}`)
  } catch (error) {
    console.error(`Failed to create optional package: ${error.message}`)
  }
}

// Verificar archivos finales
console.log('\nFinal .node files:')
const finalFiles = fs.readdirSync('.').filter((file) => file.endsWith('.node'))
finalFiles.forEach((file) => {
  const stats = fs.statSync(file)
  console.log(`  ${file} (${stats.size} bytes)`)
})

// Crear paquetes opcionales
createOptionalPackageLinks()
