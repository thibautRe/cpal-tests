import React from 'react'
import ReactDom from 'react-dom'

import Server from './comm/server'
import Oscillogram from './widgets/oscillogram'

const server = new Server(6142)

const Knob = ({ server, parameter }) => (
  <input type="range" onInput={e => {
    server.send({ type: "InstrumentMessage", data: { parameter, value: parseInt(e.target.value) / 100 } })
  }} />
)

const Controls = () => (
  <>
    <Knob server={server} parameter="Frequency" />
    <Knob server={server} parameter="Q" />
    <Oscillogram server={server} />
  </>
)

ReactDom.render(<Controls />, document.getElementById('app'))
