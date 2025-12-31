import {
  createOnMessage as __wasmCreateOnMessageForFsProxy,
  getDefaultContext as __emnapiGetDefaultContext,
  instantiateNapiModuleSync as __emnapiInstantiateNapiModuleSync,
  WASI as __WASI,
} from '@napi-rs/wasm-runtime'



const __wasi = new __WASI({
  version: 'preview1',
})

const __wasmUrl = new URL('./overlay-napi.wasm32-wasi.wasm', import.meta.url).href
const __emnapiContext = __emnapiGetDefaultContext()


const __sharedMemory = new WebAssembly.Memory({
  initial: 4000,
  maximum: 65536,
  shared: true,
})

const __wasmFile = await fetch(__wasmUrl).then((res) => res.arrayBuffer())

const {
  instance: __napiInstance,
  module: __wasiModule,
  napiModule: __napiModule,
} = __emnapiInstantiateNapiModuleSync(__wasmFile, {
  context: __emnapiContext,
  asyncWorkPoolSize: 4,
  wasi: __wasi,
  onCreateWorker() {
    const worker = new Worker(new URL('./wasi-worker-browser.mjs', import.meta.url), {
      type: 'module',
    })

    return worker
  },
  overwriteImports(importObject) {
    importObject.env = {
      ...importObject.env,
      ...importObject.napi,
      ...importObject.emnapi,
      memory: __sharedMemory,
    }
    return importObject
  },
  beforeInit({ instance }) {
    for (const name of Object.keys(instance.exports)) {
      if (name.startsWith('__napi_register__')) {
        instance.exports[name]()
      }
    }
  },
})
export default __napiModule.exports
export const Overlay = __napiModule.exports.Overlay
export const blendColors = __napiModule.exports.blendColors
export const calculateBufferSize = __napiModule.exports.calculateBufferSize
export const colorBlack = __napiModule.exports.colorBlack
export const colorBlue = __napiModule.exports.colorBlue
export const colorCyan = __napiModule.exports.colorCyan
export const colorDarkGray = __napiModule.exports.colorDarkGray
export const colorGray = __napiModule.exports.colorGray
export const colorGreen = __napiModule.exports.colorGreen
export const colorLightGray = __napiModule.exports.colorLightGray
export const colorMagenta = __napiModule.exports.colorMagenta
export const colorOrange = __napiModule.exports.colorOrange
export const colorPink = __napiModule.exports.colorPink
export const colorRed = __napiModule.exports.colorRed
export const colorToHex = __napiModule.exports.colorToHex
export const colorToRgba = __napiModule.exports.colorToRgba
export const colorToRgbHex = __napiModule.exports.colorToRgbHex
export const colorTransparent = __napiModule.exports.colorTransparent
export const colorWhite = __napiModule.exports.colorWhite
export const colorYellow = __napiModule.exports.colorYellow
export const createColor = __napiModule.exports.createColor
export const createPosition = __napiModule.exports.createPosition
export const createRgbaBuffer = __napiModule.exports.createRgbaBuffer
export const createSize = __napiModule.exports.createSize
export const drawCircle = __napiModule.exports.drawCircle
export const drawLine = __napiModule.exports.drawLine
export const drawPixel = __napiModule.exports.drawPixel
export const fillBufferColor = __napiModule.exports.fillBufferColor
export const fillBufferRgba = __napiModule.exports.fillBufferRgba
export const lerpColors = __napiModule.exports.lerpColors
export const WindowLevel = __napiModule.exports.WindowLevel
