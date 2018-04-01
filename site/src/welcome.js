import React from 'react'
import Plotly from 'plotly.js'
import Spatium from './spatium'

import Renderer from './renderer'

let SpatiumWorker = require("./spatium.worker.js");

export default class Welcome extends React.Component {
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
      fps: 30,
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
    var layout = {
      title: '',
      showlegend: true,
    };
    Plotly.newPlot(this.refs.graph, this.data, layout, { staticPlot: true })

    this.renderer = new Renderer(this.refs.canvas)
    this.renderer.clearScreen()

    this.clickStart()

    this.worker = new SpatiumWorker()
    this.worker.onmessage = event => {
      const data = JSON.parse(event.data);

      if (data.type == "log") {
        // term.write(data.message + "\r\n")
        // console.log(data.message)

      } else if (data.type == "ready") {
        this.setState((state, p) => {
          state.ready = true
          return state
        })

        this.worker.postMessage({ "type": "start" })

      } else if (data.type == "result") {

        var dataUpdated = false
        for (var i in data.result) {
          const step = data.result[i]
          if (step.hasOwnProperty("episodeResult")) {
            // console.log(step)
            this.data[0].x.push(this.data[0].x.length)
            this.data[0].y.push(step.episodeResult.score)
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
          Plotly.restyle(this.refs.graph, '', this.data)
        }

        this.steps = this.steps.concat(data.result)

      } else {
        console.log(data)
      }
    }
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
      const stepIndex = Math.min(this.state.stepIndex + 1, this.steps.length - 1)

      const step = this.steps[stepIndex]
      if (typeof step != 'undefined') {
        const renderingInfo = step.renderingInfo

        this.setState(state => {
          state.stepIndex = stepIndex
          state.step = step
          return state
        })

        if (typeof renderingInfo != 'undefined') {
          this.renderer.render(renderingInfo)
        }

        var annotations = []
        for (var i in this.annotations) {
          const ann = this.annotations[i]
          annotations.push({
            x: ann.episode,
            y: 0,
            text: ann.text,
          })
        }

        const update = {
          annotations: annotations,
          shapes: [
            {
              type: 'line',
              x0: step.episode,
              y0: 0,
              x1: step.episode,
              y1: 40,
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
    var stepHtml = ""
    const step = this.steps[this.state.stepIndex]
    if (typeof step != 'undefined') {
      stepHtml = <pre className="frame-info">
        Episode: {step.episode}<br />
        Step: {step.step}<br />
        Action: {step.action}<br />
      </pre>
    }

    var controlPanel = "Loading module..."
    if (this.state.ready) {
      controlPanel = <div>
        <div className="panel panel-default">
          <div className="panel-body">
            <div className="btn-group" role="group">
              <button className="btn btn-success" disabled={!this.state.ready || this.state.running} onClick={this.clickStart}>Start</button>
              <button className="btn btn-warning" disabled={!this.state.ready || this.state.running} onClick={this.clickStep}>Step</button>
              <button className="btn btn-danger" disabled={!this.state.ready || !this.state.running} onClick={this.clickStop}>Stop</button>
            </div>
          </div>
        </div>
        <br />
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

    const terms = <div>
      <h2>Terms</h2>
      <ul>
        <li><strong>Step</strong> - A single turn of the game</li>
        <li><strong>Episode</strong> - A complete playthrough of the game</li>
      </ul>
    </div>

    return (
      <div>
        <div className="row">

          <div className="col">
            <canvas ref="canvas" className="canvas"></canvas>
            {terms}
          </div>

          <div className="col">
            {controlPanel}
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
