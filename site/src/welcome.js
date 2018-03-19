import React from 'react'
import Plotly from 'plotly.js'
import Spatium from './spatium'
// import { Terminal } from 'xterm'
// import * as fit from 'xterm/lib/addons/fit/fit'

export default class Welcome extends React.Component {
  constructor(props) {
    super(props);
    this.clickStep = this.clickStep.bind(this);
    this.clickRun = this.clickRun.bind(this);
    this.state = { ready: false, running: false };
  }
  componentDidMount() {
    // Terminal.applyAddon(fit)
    // var term = new Terminal()
    // term.open(this.refs.terminal)
    // term.fit()

    Spatium.new(this.refs.canvas, this.refs.frameInfo, (log) => {
      // term.write(log + "\r\n")
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

    const fps = 240
    const targetDelta = 1000 / fps
    let actualFps = 0;

    let prevTimestamp = null;
    const gameLoopStep = (timestamp) => {
      if (!this.state.running) {
        return
      }

      if (!prevTimestamp) {
        prevTimestamp = timestamp
      }

      let delta = timestamp - prevTimestamp;
      // console.log(delta);

      if (delta <= targetDelta) {
        requestAnimationFrame(gameLoopStep)
        return
      }

      if (this.spatium.step()) {
        prevTimestamp = timestamp
        // actualFps = round(1000 / delta, 2)
        // console.log("fps " + actualFps)
        requestAnimationFrame(gameLoopStep)
      } else {
        console.log("Game loop ended")
      }
    }

    gameLoopStep()
  }
  render() {
    return (
      <div>
        <div className="row">
          {/* <div className="col">
            <div ref="terminal" className="terminal"></div>
          </div> */}
          </div>
        <div className="row">
          <div className="col">

            <h2>Console</h2>
            <p>Source code @
                  <a href="https://github.com/shanegibbs/spatium-wasm">shanegibbs/spatium-wasm</a>
            </p>

            <canvas ref="canvas" className="canvas"></canvas>

          </div>
          <div className="col">
            <div className="panel panel-default">
              <div className="panel-body">
                <button className="btn btn-success" disabled={!this.state.ready} onClick={this.clickRun}>Start</button>
                <button className="btn btn-danger" disabled={!this.state.ready} onClick={this.clickRun}>Stop</button>
                <button className="btn btn-warning" disabled={!this.state.ready} onClick={this.clickStep}>Step</button>
              </div>
            </div>
            <div className="progress">
              <div id="run-progress" className="progress-bar" role="progressbar" aria-valuenow="00" aria-valuemin="0" aria-valuemax="100" style={{ width: 0 }}>
              </div>
            </div>
            <p>Output</p>
            <pre ref="frameInfo" className="frame-info"></pre>
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