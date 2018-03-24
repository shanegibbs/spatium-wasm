import Spatium from './spatium'

let i = 0;
var spatium = 0;
var running = false;

console.log("Started worker")

Spatium.new(0, 0, (log) => {
  postMessage(JSON.stringify({ "type": "log", "message": log }));
}, s => {
  spatium = s
  postMessage(JSON.stringify({ "type": "ready" }));
})

function run() {
  while (true) {
    const result = spatium.step();
    postMessage(JSON.stringify({ "type": "result", "result": result }));
    if (result.done) {
      running = false
    }
    if (result.hasOwnProperty("episodeResult")) {
      // postMessage(JSON.stringify({ "type": "result", "result": result }));
      break
    }
  }
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
    start()
  } else if (e.data.type == "stop") {
    running = false
  } else if (e.data.type == "step") {
    run()
  }
}
