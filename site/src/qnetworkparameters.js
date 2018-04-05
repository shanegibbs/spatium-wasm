import React from 'react'

const parametersKey = 'parameters'

class ValueParameter extends React.Component {
  constructor(props) {
    super(props)
    this.onChange = this.onChange.bind(this)
    this.state = props.value
  }
  onChange(f) {
    f(this.state)
    this.props.onChange(this.state)
  }
  render() {
    return <div className="form-row">
      <div className="form-group col-md-4">
        <input type="text" className="form-control" placeholder="Float"
          value={this.state.initialRate}
          onChange={(e) => this.onChange(state => state.initialRate = parseFloat(e.target.value))} />
        <small id="fpsHelp" className="form-text text-muted">Initial rate</small>
      </div>
      <div className="form-group col-md-4">
        <input type="text" className="form-control" placeholder="Float"
          value={this.state.finalRate}
          onChange={(e) => this.onChange(state => state.finalRate = parseFloat(e.target.value))} />
        <small id="fpsHelp" className="form-text text-muted">Final rate</small>
      </div>
      <div className="form-group col-md-4">
        <input type="text" className="form-control" placeholder="Integer"
          value={this.state.finalEpisode}
          onChange={(e) => this.onChange(state => state.finalEpisode = parseInt(e.target.value))} />
        <small id="fpsHelp" className="form-text text-muted">At final episode</small>
      </div>
    </div>
  }
}

export default class QNetworkParameters extends React.Component {
  constructor(props) {
    super(props)
    this.onChange = this.onChange.bind(this)
    this.state = props.parameters
  }
  onChange(f) {
    f(this.state)
    this.props.onChange(this.state)
  }
  render() {
    return <div>
      <h5>General</h5>
      <div className="form-group">
        <input type="text" className="form-control" placeholder="Integer"
          value={this.props.parameters.minibatchSize}
          onChange={(e) => this.onChange(state => state.minibatchSize = parseInt(e.target.value))} />
        <small id="fpsHelp" className="form-text text-muted">Minibatch size</small>
      </div>
      <div className="form-group">
        <input type="text" className="form-control" placeholder="Integer"
          value={this.state.expierenceBufferSize}
          onChange={(e) => this.onChange(state => state.expierenceBufferSize = parseInt(e.target.value))} />
        <small id="fpsHelp" className="form-text text-muted">Expierence buffer size</small>
      </div>
      <div className="form-group">
        <input type="text" className="form-control" placeholder="Float"
          value={this.state.discountFactor}
          onChange={(e) => this.onChange(state => state.discountFactor = parseFloat(e.target.value))} />
        <small id="fpsHelp" className="form-text text-muted">Discount factor</small>
      </div>

      <h5>Learning</h5>
      <ValueParameter
        value={this.state.learning}
        onChange={(v) => this.onChange(state => state.learning = v)} />

      <h5>Exploration</h5>
      <ValueParameter
        value={this.state.exploration}
        onChange={(v) => this.onChange(state => state.exploration = v)} />

    </div>
  }
}
