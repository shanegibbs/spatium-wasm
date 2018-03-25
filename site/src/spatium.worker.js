import Spatium from './spatium'

let i = 0;
var spatium = 0;
var running = false;
var targetFps = 0;

console.log("Started worker")

Spatium.new(0, 0, (log) => {
  postMessage(JSON.stringify({ "type": "log", "message": log }));
}, s => {
  spatium = s
  postMessage(JSON.stringify({ "type": "ready" }));
})

function step() {
  const result = spatium.step();
  postMessage(JSON.stringify({ "type": "result", "result": result }));
  if (result.done) {
    running = false
  }
  return result
}

function run() {
  const result = step()
  if (running) {
    setTimeout(run, 0)
  }
}

function start() {
  running = true
  setTimeout(run, 0)
}

onmessage = function (e) {
  // console.log(e.data)
  // console.log("Received message")

  if (e.data.type == "start") {
    targetFps = e.data.fps
    start()
  } else if (e.data.type == "stop") {
    running = false
  } else if (e.data.type == "step") {
    step()
  }
}
