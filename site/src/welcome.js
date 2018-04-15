import React from 'react'
import Sim from './sim'
import QNetworkParameters from './qnetworkparameters'
import Spatium from './spatium'

const qNetwork = "QNetwork"
const qTable = "QTable"

export default class Welcome extends React.Component {
  constructor(props) {
    super(props)
    this.handleGameSelect = this.handleGameSelect.bind(this)
    this.handleModelSelect = this.handleModelSelect.bind(this)
    this.handleStartClick = this.handleStartClick.bind(this)
    this.handleCloseClick = this.handleCloseClick.bind(this)

    this.state = {
      loaded: false,
      game: 1,
      model: null,
      parameters: {},
      modelDescriptions: {},
    }

    Spatium.new((log) => {
      console.log(log)
    }, s => {
      console.log("Loaded")
      this.setState((state, p) => {
        const modelDescriptions = s.modelDescriptions()
        state.modelDescriptions = modelDescriptions

        // capture first model returned so that we can select
        // it if nothing is selected yet (state.model)
        let firstId = null;

        for (const id in state.modelDescriptions) {
          if (firstId == null) {
            firstId = id
          }
          state.parameters[id] = state.modelDescriptions[id].defaultParameters
          if (state.parameters[id] == null) {
            state.parameters[id] = {}
          }
          state.parameters[id].type = id
        }

        if (state.model == null) {
          state.model = firstId
        }

        state.loaded = true

        return state
      })
    })

  }
  handleGameSelect(e) {
    const target = e.target
    this.setState((state, p) => {
      state.game = parseInt(target.value)
      return state
    })
  }
  handleModelSelect(e) {
    const target = e.target
    this.setState((state, p) => {
      state.model = target.value
      return state
    })
  }
  handleStartClick() {
    this.setState((state, p) => {
      state.selected = {
        game: state.game,
        model: state.model,
      }
      return state
    })
  }
  handleCloseClick() {
    this.setState((state, p) => {
      state.selected = null
      return state
    })
  }
  isRunning() {
    return this.state.selected != null
  }
  render() {
    let gameDescription = <div className="col"><p>None</p></div>
    if (this.state.game == 1) {
      gameDescription = <div className="col"><p>Simple game on a 3x3 grid.</p></div>
    }

    let modelParameters = <p>No parameters</p>
    {
      const model = this.state.model
      let parameters = this.state.parameters[model]
      const update = (parameters) => {
        this.setState((state, p) => {
          state.parameters[model] = parameters
          // console.log(state)
          return state
        })
      }
      if (model == qNetwork) {
        modelParameters = <QNetworkParameters parameters={parameters} onChange={update} />
      }
    }

    const gameOptions = <div className="col">
      <h3>Game</h3>
      <select className="custom-select" defaultValue={this.state.game} onChange={this.handleGameSelect}>
        <option value="1">Game 1</option>
      </select>
      {gameDescription}
    </div>

    let modelSelectOptions = []
    for (const i in this.state.modelDescriptions) {
      const modelDescription = this.state.modelDescriptions[i]
      const id = modelDescription.id
      const name = modelDescription.name
      modelSelectOptions.push(<option key={id} value={id}>{name}</option>)
    }

    const modelOptions = <div className="col">
      <h3>Model</h3>
      <select className="custom-select" defaultValue={this.state.model} onChange={this.handleModelSelect}>
        {modelSelectOptions}
      </select>
      <br />
      <br />
      <h4>Parameters</h4>
      {modelParameters}
    </div>

    const startButton = <div className="col text-right">
      <button className="btn btn-primary" onClick={this.handleStartClick}>Start</button>
    </div>

    const stopButton = <div className="col text-right">
      <button className="btn btn-danger" onClick={this.handleCloseClick}>Back</button>
    </div>

    let game = gameOptions
    let model = modelOptions
    let button = startButton
    let sim = <div />

    if (this.isRunning()) {
      game = ''
      model = ''
      button = stopButton
      sim = <Sim modelParameters={this.state.parameters[this.state.model]} />
    }

    const terms = <div>
      <h4>Terms</h4>
      <ul>
        <li><strong>Step</strong> - A single turn of the game</li>
        <li><strong>Episode</strong> - A complete playthrough of the game</li>
      </ul>
    </div>

    let page = <div>
      <div className="row">
        {game}
        {model}
        {button}
      </div>
      {sim}
      <hr />
      <div>
        {terms}
      </div>
    </div>

    if (this.state.loaded) {
      return page
    }

    return <div></div>
  }
}
