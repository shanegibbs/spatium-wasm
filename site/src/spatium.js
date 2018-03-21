const maxEpisodes = 3000
const renderOn = true

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

  canvas.width = (gridStepWidth * gridWidth) + (gridOffsetX * 2);
  canvas.style.width = canvas.width + "px";
  canvas.height = (gridStepHeight * gridHeight) + (gridOffsetY * 2);
  canvas.style.height = canvas.height + "px";

  function sp_print(text) {
    text = stringFrom(spatium, text)
    // console.info("> " + text)
    logger(text)
  }
  function sp_random() {
    return Math.random()
  }

  const ctx = canvas.getContext("2d");

  // Draw crisp lines
  // http://www.mobtowers.com/html5-canvas-crisp-lines-every-time/
  ctx.translate(0.5, 0.5)

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
    info = stringFrom(spatium, info)
    frameInfo.innerHTML = info
  }
  function sp_episode_number(i) {
    const valeur = (i / maxEpisodes) * 100
    // $('.progress-bar').css('width', valeur + '%').attr('aria-valuenow', valeur)
  }
  function expf(a) {
    // console.log("call expf")
    return Math.exp(a)
  }
  function logf(a) {
    // console.log("call logf")
    return Math.log(a)
  }
  function Math_tanh(a) {
    // console.log("call Math_tanh")
    return Math.tanh(a)
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
    Math_tanh,
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
    spatium.ping = instance.exports.ping
    console.log("Ping: " + spatium.ping())
    spatium.memory = instance.exports.memory
    spatium.alloc = instance.exports.alloc
    spatium.dealloc = instance.exports.dealloc

    spatium.setup = instance.exports.setup
    spatium.step = instance.exports.step

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
