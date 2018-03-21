import React from 'react'
import Plotly from 'plotly.js'
import Spatium from './spatium'
import { Terminal } from 'xterm'
import * as fit from 'xterm/lib/addons/fit/fit'

export default class Welcome extends React.Component {
  constructor(props) {
    super(props)
    this.clickStep = this.clickStep.bind(this)
    this.clickRun = this.clickRun.bind(this)
    this.handleFpsChange = this.handleFpsChange.bind(this)

    this.state = { ready: false, running: false, fps: 30 }
  }
  componentDidMount() {
    Terminal.applyAddon(fit)
    var term = new Terminal()
    term.open(this.refs.terminal)
    term.fit()

    Spatium.new(this.refs.canvas, this.refs.frameInfo, (log) => {
      term.write(log + "\r\n")
    }, spatium => {
      spatium.step()
      this.spatium = spatium
      this.setState((state, p) => {
        state.ready = true
        return state
      })
    })

    var trace1 = {
      x: [1, 2, 3, 4],
      y: [10, 15, 13, 17],
      type: 'scatter'
    };

    var trace2 = {
      x: [1, 2, 3, 4],
      y: [16, 5, 11, 9],
      type: 'scatter'
    };

    var data = [trace1, trace2];

    Plotly.newPlot(this.refs.graph, data);

    data[0].x.push(5)
    data[0].y.push(20)

    data[1].x.push(5)
    data[1].y.push(-5)

    Plotly.restyle(this.refs.graph, '', data)
  }
  clickStep() {
    this.spatium.step()
  }
  clickRun() {

    if (this.state.running) {
      this.setState({ running: false })
      return
    } else {
      this.state.running = true
      this.setState({ running: true })
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
      if (fps == 0) {
        var i = 10
        while (i-- > 0 && this.spatium.step()) { }
        requestAnimationFrame(gameLoopStep)
        return
      }
      const targetDelta = 1000 / fps

      let delta = timestamp - prevTimestamp;
      // console.log(delta);

      if (delta <= targetDelta) {
        requestAnimationFrame(gameLoopStep)
        return
      }

      if (this.spatium.step()) {
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

            <p>Source code @
                  <a href="https://github.com/shanegibbs/spatium-wasm">shanegibbs/spatium-wasm</a>
            </p>

            <canvas ref="canvas" className="canvas"></canvas>

          </div>

          <div className="col-8">
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
                  <button className="btn btn-danger" disabled={!this.state.ready || !this.state.running} onClick={this.clickRun}>Stop</button>
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
            <pre ref="frameInfo" className="frame-info"></pre>
          </div>

          <div className="col-8">
            <div ref="terminal"></div>
          </div>

        </div>
      </div>
    );
  }
}