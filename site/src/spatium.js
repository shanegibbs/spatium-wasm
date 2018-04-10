const { DateTime } = require('luxon');

const maxEpisodes = 300
let renderOn = true

const Spatium = {}

function stringFrom(module, cstr) {
  // console.log("Module memory size: " + module.memory.buffer.byteLength / 1024 + " KB")
  let sArr = []
  let i = 0
  let memOffset = cstr
  const bufferSize = 1024

  for (; ;) {
    const bytes = new Uint8Array(module.memory.buffer).slice(memOffset, memOffset + bufferSize)
    memOffset += bufferSize

    for (let n in bytes) {
      let b = bytes[n]
      if (b == 0) {
        module.dealloc(cstr)
        return sArr.join('')
      }
      // using index is faster than push() here
      sArr[i++] = String.fromCharCode(b)
    }
  }
}

function env(spatium, logger) {

  function sp_print(text) {
    text = stringFrom(spatium, text)
    // console.info("> " + text)
    logger(text)
  }
  function sp_random() {
    return Math.random()
  }

  function expf(a) {
    return Math.exp(a)
  }
  function logf(a) {
    return Math.log(a)
  }
  function powf(a) {
    return Math.pow(a)
  }
  function Math_tanh(a) {
    return Math.tanh(a)
  }
  function Math_atan(a) {
    return Math.atan(a)
  }
  function Math_log1p(a) {
    return Math.log1p(a)
  }
  function Math_sinh(a) {
    return Math.sinh(a)
  }
  function Math_asin(a) {
    return Math.asin(a)
  }
  function cosf(a) {
    return Math.cosf(a)
  }
  function Math_cosh(a) {
    return Math.cosh(a)
  }
  function sinf(a) {
    return Math.sinf(a)
  }
  function Math_tan(a) {
    return Math.tan(a)
  }
  function Math_acos(a) {
    return Math.acos(a)
  }

  let imports = {
    sp_print,
    sp_random,
    expf,
    logf,
    powf,
    Math_tanh,
    Math_atan,
    Math_log1p,
    Math_sinh,
    Math_asin,
    cosf,
    Math_cosh,
    sinf,
    Math_tan,
    Math_acos,
  }
  return { env: imports }
}

function fetchAndBuild(url, importObject) {
  return fetch(url).then(response =>
    response.arrayBuffer()
  ).then(bytes => {
    Spatium.bytes = bytes
    return WebAssembly.instantiate(bytes, importObject)
  }
  ).then(results =>
    results.instance
  ).then(buildModule);
}

function buildModule(instance) {
  Spatium.memory = instance.exports.memory
  Spatium.alloc = instance.exports.alloc
  Spatium.dealloc = instance.exports.dealloc
  Spatium.setup = instance.exports.setup
  Spatium.step = instance.exports.step
  return Spatium
}

const wasmBytes = fetch("spatium_wasm.wasm").then(response =>
  response.arrayBuffer()
)

function newString(module, str) {
  const utf8Encoder = new TextEncoder("UTF-8");
  let string_buffer = utf8Encoder.encode(str)
  let len = string_buffer.length
  let ptr = module.alloc(len + 1)

  let memory = new Uint8Array(module.memory.buffer); // TODO: this is probably very slow
  for (let i = 0; i < len; i++) {
    memory[ptr + i] = string_buffer[i]
  }

  memory[ptr + len] = 0;

  return ptr;
}

Spatium.new = (logger, readyCallback) => {
  const spatium = {}

  wasmBytes.then(bytes => WebAssembly.instantiate(bytes, env(spatium, logger))
  ).then(results => {
    const instance = results.instance
    spatium.memory = instance.exports.memory
    spatium.alloc = instance.exports.alloc
    spatium.dealloc = instance.exports.dealloc

    spatium.setup = (model_params, max_episodes) => {
      const model_params_buf = newString(instance.exports, model_params)
      const result = instance.exports.setup(model_params_buf, max_episodes)
      spatium.dealloc(model_params_buf)
      return stringFrom(spatium, result)
    }
    spatium.step = (count) => {
      return JSON.parse(stringFrom(spatium, instance.exports.step(count)))
    }
    spatium.modelDescriptions = () => {
      return JSON.parse(stringFrom(spatium, instance.exports.model_descriptions()))
    }
    spatium.version = instance.exports.version

    const version = spatium.version();
    const date = DateTime.fromMillis(version * 1000)
    const age = date.diffNow(["days", "hours", "minutes"])
    console.log("Version: " + version)
    console.log("Version: " + date.toISO())
    console.log("Version: " + age.get("hour") + "h " + Math.round(age.get("minute") * -1) + "m")

    logger("[system] Loaded spatium module version " + date.toISO())

    readyCallback(spatium)
  })

  // results.then(results => {
  //   console.log(results)
  //   const instance = results.instance
  //   // spatium.step = instance.exports.step
  // })

  return spatium
}

export default Spatium
