var { DateTime } = require('luxon');

const maxEpisodes = 3000
var renderOn = true

const Spatium = {}

function stringFrom(module, cstr) {
  const bytes = new Uint8Array(module.memory.buffer).slice(cstr)
  var s = ""
  for (let n in bytes) {
    var b = bytes[n]
    if (b == 0) {
      break
    }
    s += String.fromCharCode(b)
  }
  module.dealloc(cstr)
  return s
}

function env(spatium, canvas, frameInfo, logger) {
  const gridHeight = 3
  const gridWidth = 3
  const gridOffsetX = 20
  const gridOffsetY = 20
  const gridStepHeight = 60
  const gridStepWidth = 60

  var ctx = 0;
  if (canvas != 0) {
    canvas.width = (gridStepWidth * gridWidth) + (gridOffsetX * 2);
    canvas.style.width = canvas.width + "px";
    canvas.height = (gridStepHeight * gridHeight) + (gridOffsetY * 2);
    canvas.style.height = canvas.height + "px";
    ctx = canvas.getContext("2d");

    // Draw crisp lines
    // http://www.mobtowers.com/html5-canvas-crisp-lines-every-time/
    ctx.translate(0.5, 0.5)

  } else {
    renderOn = false
  }

  function sp_print(text) {
    text = stringFrom(spatium, text)
    // console.info("> " + text)
    logger(text)
  }
  function sp_random() {
    return Math.random()
  }

  function sp_clear_screen() {
    if (!renderOn) {
      return
    }
    // console.info("> clear screen")

    // clear
    ctx.fillStyle = "white"
    ctx.fillRect(0, 0, canvas.width, canvas.height)

    // draw grid
    ctx.beginPath()
    ctx.moveTo(gridOffsetX, gridOffsetY)
    ctx.lineTo(gridOffsetX, gridOffsetY + (gridStepHeight * gridHeight))
    ctx.lineTo(gridOffsetX + (gridStepWidth * gridWidth), gridOffsetY + (gridStepHeight * gridHeight))
    ctx.lineTo(gridOffsetX + (gridStepWidth * gridWidth), gridOffsetY)
    ctx.lineTo(gridOffsetX, gridOffsetY)
    ctx.strokeStyle = "black"
    ctx.lineWidth = 1

    for (let x = 1; x < gridWidth; x++) {
      ctx.moveTo(gridOffsetX + (gridStepWidth * x), gridOffsetY)
      ctx.lineTo(gridOffsetX + (gridStepWidth * x), gridOffsetY + gridStepHeight * gridHeight)
    }
    for (let y = 1; y < gridHeight; y++) {
      ctx.moveTo(gridOffsetX, gridOffsetY + gridStepHeight * y)
      ctx.lineTo(gridOffsetX + (gridStepWidth * gridWidth), gridOffsetY + gridStepHeight * y)
    }

    ctx.stroke()
  }

  function sp_draw_sprite(i, x, y) {
    if (!renderOn) {
      return
    }

    if (i == 0) {
      ctx.fillStyle = "blue"
    } else if (i == 1) {
      ctx.fillStyle = "black"
    } else if (i == 2) {
      ctx.fillStyle = "green"
    }

    ctx.fillRect(
      gridOffsetX + gridStepWidth * x,
      gridOffsetY + gridStepHeight * y,
      gridStepWidth, gridStepHeight)

    ctx.strokeStyle = "black"
    ctx.lineWidth = 1
    ctx.rect(gridOffsetX + gridStepWidth * x, gridOffsetY + gridStepHeight * y, gridStepWidth, gridStepHeight);
    ctx.stroke()
  }

  function sp_frame_info(info) {
    if (frameInfo == 0) {
      return
    }
    info = stringFrom(spatium, info)
    frameInfo.innerHTML = info
  }

  function sp_episode_number(i) {
    const valeur = (i / maxEpisodes) * 100
    // $('.progress-bar').css('width', valeur + '%').attr('aria-valuenow', valeur)
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
    sp_clear_screen,
    sp_draw_sprite,
    sp_frame_info,
    sp_episode_number,
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
  Spatium.get_charstar = function () {
    return stringFrom(Spatium, instance.exports.get_charstar())
  }
  Spatium.setup = instance.exports.setup
  Spatium.step = instance.exports.step
  Spatium.eval = instance.exports.eval
  return Spatium
}

// fetchAndBuild("spatium_wasm.wasm", env(canvas, frameInfo)).then(spatium => {
//   spatium.setup(maxEpisodes)
//   spatium.step()
// })

const wasmBytes = fetch("spatium_wasm.wasm").then(response =>
  response.arrayBuffer()
)

Spatium.new = (canvas, frameInfo, logger, readyCallback) => {
  const spatium = {}

  wasmBytes.then(bytes => WebAssembly.instantiate(bytes, env(spatium, canvas, frameInfo, logger))
  ).then(results => {
    const instance = results.instance
    spatium.memory = instance.exports.memory
    spatium.alloc = instance.exports.alloc
    spatium.dealloc = instance.exports.dealloc

    spatium.setup = instance.exports.setup
    spatium.step = () => {
      return JSON.parse(stringFrom(spatium, instance.exports.step()))
    }
    spatium.version = instance.exports.version

    const version = spatium.version();
    const date = DateTime.fromMillis(version * 1000)
    const age = date.diffNow(["days", "hours", "minutes"])
    console.log("Version: " + version)
    console.log("Version: " + date.toISO())
    console.log("Version: " + age.get("hour") + "h " +  Math.round(age.get("minute") * -1) + "m")

    logger("[system] Loaded spatium module version " + date.toISO())

    spatium.setup(maxEpisodes)

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
