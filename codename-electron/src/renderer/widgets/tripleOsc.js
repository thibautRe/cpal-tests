import React from 'react'

const FREQUENCIES = [
  'Sine', 'Square', 'Sawtooth', 'Triangle'
];

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
    const { server } = this.props

    console.log(oscillators);


    return oscillators.map((osc, index) => (
      <div style={{ background: '#eee', padding: 5, margin: 10 }} key={index}>
        <h1>Oscillator {index + 1}</h1>
        <div>
          <select defaultValue={osc.shape} onChange={e => {
            server.send({
              type: 'InstrumentSetParameter',
              data: ['triple_osc', {
                parameter: `osc ${index + 1} shape`,
                value: e.target.value,
              }],
            })
          }}>
            {FREQUENCIES.map(freq => (
              <option id={freq} key={freq}>{freq}</option>
            ))}
          </select>
        </div>
        <label>
          Frequency:
          <input defaultValue={osc.frequency} onChange={e => {
            server.send({
              type: 'InstrumentSetParameter',
              data: ['triple_osc', {
                parameter: `osc ${index + 1} frequency`,
                value: parseInt(e.target.value) || 0,
              }],
            })
          }} />
        </label>
      </div>
    ))
  }
}