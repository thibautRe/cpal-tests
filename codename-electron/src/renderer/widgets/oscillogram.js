import React from 'react'

export default class Oscillogram extends React.Component {
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
    this.props.server.send({ type: "RootDataMessage", data: { target: 'OutputBuffer' } }).then(data => {
      this.drawBufferOutput(data)
      setTimeout(this.loop, 50)
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