import React from 'react'
import ReactDom from 'react-dom'
import net from 'net'
import readline from 'readline'

class Server {
  constructor(port) {
    this.socket = new net.Socket()
    this.socket.setEncoding('utf8')
    this.socket.connect(port)
    const rl = readline.createInterface(this.socket, this.socket)
    rl.on('line', this.onLine.bind(this))

    this.callbacks = {}
    this.callbackId = 0
  }

  onLine(line) {
    const data = JSON.parse(line)
    if (this.callbacks[data.id]) {
      this.callbacks[data.id](data.payload)
    }
  }

  send(payload) {

    return new Promise(r => {
      const { callbackId } = this
      this.callbacks[callbackId] = data => {
        delete this.callbacks[callbackId]
        r(data)
      }
      this.socket.write(`${JSON.stringify({
        id: callbackId,
        payload,
      })}\n`)
      this.callbackId++
    })
  }
}

class Canvas extends React.Component {
  constructor(props) {
    super(props)
    this.canvas = React.createRef()

    this.loop = this.loop.bind(this)
  }

  componentDidMount() {
    if (this.canvas.current) {
      this.ctx = this.canvas.current.getContext('2d');
      this.loop()
    }
  }

  loop() {
    this.props.server.send({ RootDataMessage: { target: 'OutputBuffer' } }).then(data => {
      this.drawBufferOutput(data)
      requestAnimationFrame(this.loop)
    })
  }

  getCanvasCoord(pointNr, amplitude, totalPoints) {
    return [
      pointNr / totalPoints * this.canvas.current.clientWidth,
      this.canvas.current.clientHeight * (1 + amplitude / 10) / 2,
    ]
  }

  drawBufferOutput(data) {
    const canvas = this.canvas.current
    // Erase any previous line
    this.ctx.clearRect(0, 0, canvas.clientWidth, canvas.clientHeight)

    // Draw the baseline
    this.ctx.beginPath()
    this.ctx.lineWidth = 0.5
    this.ctx.strokeStyle = '#92c9dd'
    this.ctx.moveTo(0, canvas.clientHeight / 2)
    this.ctx.lineTo(canvas.clientWidth, canvas.clientHeight / 2);
    this.ctx.stroke()

    // Draw the rest
    this.ctx.beginPath()
    this.ctx.lineWidth = 1;
    this.ctx.strokeStyle = "#333";
    this.ctx.moveTo(...this.getCanvasCoord(0, data[0], data.length))

    data.slice(1).forEach((amplitude, pointNr) =>
      this.ctx.lineTo(...this.getCanvasCoord(pointNr, amplitude, data.length))
    )

    this.ctx.stroke()
  }

  render() {
    return <canvas height={200} width={200} ref={this.canvas} />
  }
}

const server = new Server(6142)

const Knob = ({ server, parameter }) => (
  <input type="range" onInput={e => {
    server.send({ InstrumentMessage: { parameter, value: parseInt(e.target.value) / 100 } })
  }} />
)

const Controls = () => (
  <>
    <Knob server={server} parameter="Frequency" />
    <Knob server={server} parameter="Q" />
    <Canvas server={server} />
  </>
)

ReactDom.render(<Controls />, document.getElementById('app'))
