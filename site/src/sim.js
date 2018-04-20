import React from 'react'
import Plotly from 'plotly.js'
import Spatium from './spatium'

import Renderer from './renderer'

let SpatiumWorker = require("./spatium.worker.js");

export default class Sim extends React.Component {
  constructor(props) {
    super(props)
    this.clickStep = this.clickStep.bind(this)
    this.clickStart = this.clickStart.bind(this)
    this.clickStop = this.clickStop.bind(this)
    this.handleFpsChange = this.handleFpsChange.bind(this)
    this.renderLoop = this.renderLoop.bind(this)

    this.state = {
      ready: false,
      running: false,
      fps: 10,
      stepIndex: 0,
      totalSteps: 0
    }

    this.steps = []
    this.annotations = []
  }
  componentDidMount() {

    this.data = [{
      x: [],
      y: [],
      type: 'scatter',
      name: 'Episode Score'
    }]
    let layout = {
      title: '',
      showlegend: true,
    };
    Plotly.newPlot(this.refs.graph, this.data, layout, { staticPlot: true })
    this.raw_data = []

    this.renderer = new Renderer(this.refs.canvas)
    this.renderer.clearScreen()

    this.clickStart()

    this.worker = new SpatiumWorker()
    this.worker.onmessage = event => {
      if (!this.state.running) {
        return
      }

      const data = JSON.parse(event.data);

      if (data.type == "log") {
        // term.write(data.message + "\r\n")
        console.log(data.message)

      } else if (data.type == "error") {
        console.log("Error from worker: " + data.message)
        this.setState((state, p) => {
          state.error = data.message
          return state
        })

      } else if (data.type == "loaded") {
        this.setState((state, p) => {
          state.ready = false
          return state
        })
        this.worker.postMessage({
          type: "parameters",
          model: this.props.modelParameters
        })

      } else if (data.type == "ready") {
        this.setState((state, p) => {
          state.ready = true
          return state
        })
        this.worker.postMessage({ "type": "start" })

      } else if (data.type == "result") {

        let dataUpdated = false
        for (let i in data.result) {
          const step = data.result[i]
          if (step.hasOwnProperty("episodeResult")) {
            // console.log(step)
            // this.data[0].x.push(this.data[0].x.length)
            // this.data[0].y.push(step.episodeResult.score)
            this.raw_data.push(step.episodeResult.score)
            dataUpdated = true
          }
          if (step.hasOwnProperty("metrics") && step.metrics != null) {
            for (i in step.metrics.annotations) {
              const annotation = step.metrics.annotations[i]
              this.annotations.push({
                episode: step.episode,
                text: annotation,
              })
            }
          }
        }

        if (dataUpdated) {
          const maxPoints = 100
          const stepSize = this.raw_data.length / maxPoints
          this.stepSize = stepSize
          let data_x = []
          let data_y = []
          for (let i = 0; i < maxPoints; i++) {
            const offset = parseInt(i * stepSize)

            data_x.push(i)

            let valueCount = 0
            let value = 0
            for (let n = offset; n < offset + stepSize; n++) {
              value += this.raw_data[n]
              valueCount++
            }
            data_y.push(value / valueCount)

          }
          this.data[0].x = data_x
          this.data[0].y = data_y
          Plotly.restyle(this.refs.graph, '', this.data)
        }

        this.steps = this.steps.concat(data.result)

      } else {
        console.log(data)
      }
    }
  }
  componentWillUnmount() {
    this.state.running = false
    this.worker.terminate()
  }
  clickStep() {
    this.worker.postMessage({ "type": "step" })
  }
  clickStop() {
    this.worker.postMessage({ "type": "stop" })
    this.setState((state, p) => {
      state.running = false
      return state
    })
  }
  clickStart() {
    this.state.running = true
    this.setState((state, p) => {
      state.running = true
      return state
    })
    this.renderLoop()
  }
  renderLoop() {
    if (!this.state.running) {
      return
    }

    let actualFps = 0;

    let prevTimestamp = null;
    const gameLoopStep = (timestamp) => {
      if (!this.state.running) {
        return
      }

      if (!prevTimestamp) {
        prevTimestamp = timestamp
      }

      const fps = this.state.fps
      const targetDelta = 1000 / fps

      let delta = timestamp - prevTimestamp;

      // not time for next frame yet
      if (delta <= targetDelta) {
        requestAnimationFrame(gameLoopStep)
        return
      }

      // new stepIndex
      let stepIndex = Math.min(this.state.stepIndex + 1, this.steps.length - 1)

      const step = this.steps[stepIndex]
      if (typeof step != 'undefined') {
        // console.log(step)
        const renderingInfo = step.renderingInfo

        if (step.hasOwnProperty("episodeResult")) {
          const idealMin = parseInt((this.steps.length / 6) * 5)
          if (stepIndex < idealMin) {
            stepIndex = idealMin
          }
        }

        this.setState(state => {
          state.stepIndex = stepIndex
          state.step = step
          return state
        })

        if (typeof renderingInfo != 'undefined') {
          this.renderer.render(renderingInfo)
        }

        let annotations = []
        for (let i in this.annotations) {
          const ann = this.annotations[i]
          annotations.push({
            x: ann.episode,
            y: 0,
            text: ann.text,
          })
        }

        const update = {
          annotations: [],
          shapes: [
            {
              type: 'line',
              x0: step.episode / this.stepSize,
              y0: 0,
              x1: step.episode / this.stepSize,
              y1: 10,
              line: {
                color: 'rgb(255, 0, 0)',
              }
            }
          ]
        }
        Plotly.relayout(this.refs.graph, update)

      }

      // schedule again if still running
      if (this.state.running) {
        prevTimestamp = timestamp
        actualFps = Math.round(1000 / delta, 2)
        // console.log("fps " + actualFps)
        requestAnimationFrame(gameLoopStep)
      } else {
        console.log("Game loop ended")
      }
    }

    gameLoopStep()
  }
  handleFpsChange(e) {
    const newValue = e.target.value
    this.setState((state, p) => {
      state.fps = newValue
      return state
    })
  }
  render() {
    let stepHtml = ""
    const step = this.steps[this.state.stepIndex]
    if (typeof step != 'undefined') {
      stepHtml = <pre className="frame-info">
        Episode: {step.episode}<br />
        Step: {step.step}<br />
        Action: {step.action}<br />
      </pre>
    }

    let controlPanel = "Loading module..."
    if (this.state.ready) {
      controlPanel = <div>
        <div className="form-group">
          <input type="text" className="form-control" placeholder="FPS" value={this.state.fps} onChange={this.handleFpsChange} />
          <small id="fpsHelp" className="form-text text-muted">Frames per second. 0 for max.</small>
        </div>
        {stepHtml}
        <pre>
          Rendering: {this.state.stepIndex} / {this.steps.length}<br />
        </pre>
      </div>
    }

    if (typeof this.state.error != "undefined") {
      controlPanel = <div>Error:<pre>{this.state.error}</pre></div>
    }

    return (
      <div className="sim">
        <div className="row">

          <div className="col">
            <canvas ref="canvas" className="canvas"></canvas>
          </div>

          <div className="col">
            {controlPanel}
          </div>

          <div className="col">
          </div>
        </div>

        <div className="row">
          <div className="col">
            <div ref="graph"></div>
          </div>
        </div>
      </div>
    );
  }
}
