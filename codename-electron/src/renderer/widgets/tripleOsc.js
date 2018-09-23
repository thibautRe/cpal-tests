import React from 'react'

export default class extends React.Component {
  constructor(props) {
    super(props)
    this.state = { oscillators: [] }
  }

  async componentDidMount() {
    const { server } = this.props
    const state = await server.send({
      type: "InstrumentGetState",
      data: "triple_osc",
    })

    this.setState({ oscillators: state })
  }

  render() {
    const { oscillators } = this.state

    return oscillators.map((osc, index) => (
      <div style={{ background: '#eee', padding: 5, margin: 10 }} key={index}>
        <h1>Oscillator {index}</h1>
        <div>{osc.shape}</div>
        <label>
          Frequency:
          <input defaultValue={osc.frequency} />
        </label>
      </div>
    ))
  }
}