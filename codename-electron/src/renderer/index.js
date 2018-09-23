import React from 'react'
import ReactDom from 'react-dom'

import Server from './comm/server'
import Oscillogram from './widgets/oscillogram'
import TripleOsc from './widgets/tripleOsc'

const server = new Server(6142)

class Knob extends React.Component {
  constructor(props) {
    super(props)
    this.state = { value: 0 }
  }

  async componentDidMount() {
    const { server, parameter } = this.props

    const value = await server.send({
      type: "InstrumentGetState",
    })

    console.log(value);


    this.setState({ value: value * 100 })
  }

  render() {
    const { server, parameter } = this.props

    return (
      <input
        type="range"
        value={this.state.value}
        onInput={e => {
          server.send({
            type: "InstrumentSetValue",
            data: { parameter, value: parseInt(e.target.value) / 100 }
          })
        }}
        onChange={e => {
          this.setState({ value: e.target.value })
        }}
      />
    )
  }
}

const Controls = () => (
  <>
    {/* <Knob server={server} parameter="Frequency" />
    <Knob server={server} parameter="Q" /> */}
    <TripleOsc server={server} />
    <Oscillogram server={server} />
  </>
)

ReactDom.render(<Controls />, document.getElementById('app'))
