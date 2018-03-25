import React from 'react'
import Plotly from 'plotly.js'
import Spatium from './spatium'
import { Terminal } from 'xterm'
import * as fit from 'xterm/lib/addons/fit/fit'

let SpatiumWorker = require("./spatium.worker.js");


export default class Welcome extends React.Component {
  constructor(props) {
    super(props)
    this.clickStep = this.clickStep.bind(this)
    this.clickRun = this.clickRun.bind(this)
    this.clickStop = this.clickStop.bind(this)
    this.handleFpsChange = this.handleFpsChange.bind(this)
    this.onChangeRender = this.onChangeRender.bind(this)
    this.renderLoop = this.renderLoop.bind(this)

    this.state = { ready: false, running: false, fps: 10, render: true }

  }
  componentDidMount() {
    Terminal.applyAddon(fit)
    var term = new Terminal()
    term.open(this.refs.terminal)
    term.fit()

    this.data = [{
      x: [],
      y: [],
      type: 'scatter',
    }]
    Plotly.newPlot(this.refs.graph, this.data)

    const gridHeight = 3
    const gridWidth = 3
    const gridOffsetX = 20
    const gridOffsetY = 20
    const gridStepHeight = 60
    const gridStepWidth = 60

    const canvas = this.refs.canvas
    canvas.width = (gridStepWidth * gridWidth) + (gridOffsetX * 2)
    canvas.style.width = canvas.width + "px"
    canvas.height = (gridStepHeight * gridHeight) + (gridOffsetY * 2)
    canvas.style.height = canvas.height + "px"

    const ctx = canvas.getContext("2d")

    // Draw crisp lines
    // http://www.mobtowers.com/html5-canvas-crisp-lines-every-time/
    ctx.translate(0.5, 0.5)

    function clear_screen() {
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

    function draw_sprite(i, x, y) {
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

    this.clear_screen = clear_screen
    this.draw_sprite = draw_sprite
    this.forRender = 0

    this.clear_screen()

    this.worker = new SpatiumWorker()
    this.worker.onmessage = event => {
      const data = JSON.parse(event.data);

      if (data.type == "log") {
        term.write(data.message + "\r\n")

      } else if (data.type == "ready") {
        this.setState((state, p) => {
          state.ready = true
          return state
        })

      } else if (data.type == "result") {
        const step = data.result

        // console.log(step)
        this.setState((state, p) => {
          state.episode = step.episode
          state.step = step.step
          state.action = step.action
          return state
        })

        if (step.hasOwnProperty("renderingInfo")) {
          const renderingInfo = step.renderingInfo
          this.forRender = renderingInfo
          // console.log(renderingInfo)
          // draw_sprite(0, renderingInfo.x, renderingInfo.y)
          // draw_sprite(1, 1, 1)
          // draw_sprite(2, 2, 2)
        }

        if (step.hasOwnProperty("episodeResult")) {
          // console.log(step)
          this.data[0].x.push(this.data[0].x.length)
          this.data[0].y.push(step.episodeResult.score)
          Plotly.restyle(this.refs.graph, '', this.data)
        }

      } else {
        console.log(data)
      }
    }
  }
  onChangeRender() {
    const renderNew = !this.state.render

    let state = this.state
    state.render = renderNew
    this.setState(state)

    if (!this.state.running) {
      return
    }

    this.renderLoop()
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
  clickRun() {
    this.state.running = true
    this.setState({ running: true })
    this.renderLoop()
  }
  renderLoop() {
    if (!this.state.render) {
      this.worker.postMessage({ "type": "start" })
      return
    }
    this.worker.postMessage({ "type": "stop" })
    this.worker.postMessage({ "type": "step" })

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

      const forRender = this.forRender
      if (forRender != 0) {
        this.worker.postMessage({ "type": "step" })

        // console.log(renderingInfo)
        this.clear_screen()
        this.draw_sprite(1, 1, 1)
        this.draw_sprite(2, 2, 2)
        this.draw_sprite(0, forRender.x, forRender.y)
      }

      // schedule again if still running
      if (this.state.running && this.state.render) {
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
    console.log(newValue)
    this.setState((state, p) => {
      state.fps = newValue
      return state
    })
  }
  render() {
    return (
      <div>
        <div className="row">
          <div className="col">
            <div ref="terminal"></div>
          </div>
        </div>
        <div className="row">
          <div className="col">

            <canvas ref="canvas" className="canvas"></canvas>
            <div>
              <h2>Terms</h2>
              <ul>
                <li><strong>Step</strong> - A single turn of the game</li>
                <li><strong>Episode</strong> - A complete playthrough of the game</li>
              </ul>
            </div>

          </div>

          <div className="col-8">
            {/* <h4>Steps to hit target</h4> */}
            {/* <p>(lower is better)</p> */}
            <div ref="graph"></div>
          </div>

        </div>

        <div className="row">

          <div className="col">
            <div className="panel panel-default">
              <div className="panel-body">
                <div className="btn-group" role="group">
                  <button className="btn btn-success" disabled={!this.state.ready || this.state.running} onClick={this.clickRun}>Start</button>
                  <button className="btn btn-warning" disabled={!this.state.ready || this.state.running} onClick={this.clickStep}>Step</button>
                  <button className="btn btn-danger" disabled={!this.state.ready || !this.state.running} onClick={this.clickStop}>Stop</button>
                </div>

                <div className="form-check">
                  <input className="form-check-input" type="checkbox" checked={this.state.render} onChange={this.onChangeRender} />
                  <label className="form-check-label">
                    Render
                  </label>
                </div>

              </div>
            </div>

            <br />

            <div className="form-group">
              <input type="text" className="form-control" placeholder="FPS" value={this.state.fps} onChange={this.handleFpsChange} />
              <small id="fpsHelp" className="form-text text-muted">Frames per second. 0 for max.</small>
            </div>

            {/* <div className="progress">
              <div id="run-progress" className="progress-bar" role="progressbar" aria-valuenow="00" aria-valuemin="0" aria-valuemax="100" style={{ width: 0 }}>
              </div>
            </div> */}
            <pre ref="frameInfo" className="frame-info">
              Episode: {this.state.episode}<br />
              Step: {this.state.step}<br />
              Action: {this.state.action}<br />
            </pre>
          </div>

          <div className="col-8">
            <div ref="terminal"></div>
          </div>

        </div>
      </div>
    );
  }
}