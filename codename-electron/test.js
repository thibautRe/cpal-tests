const net = require('net')

const socket = new net.Socket()
socket.setEncoding('utf8')
socket.connect(6142)
socket.on('error', err => console.error(err))

let bufferedData = ''
const fixData = newData => {
  const splittedData = (bufferedData + newData).split('\n').filter(Boolean)
  // Case 1: the TCP socket has a newline character at the end of the buffer
  if (newData[newData.length - 1] === '\n') {
    bufferedData = ''
    return splittedData
  }
  // Case 2: The TCP socket doesn't have a newline char but there is a newline
  // char somewhere in the current buffer
  if (splittedData.length > 1) {
    bufferedData = splittedData.splice(splittedData.length - 1)
    return splittedData
  }

  // Case 3: The TCP socket doesn't have any newline char
  bufferedData += newData
  return []
}
socket.on('data', data => {
  data = fixData(data)

  data.map(JSON.parse).forEach(data => {
    // In that case, it's an array so let's show it in the canvas
    if (!data.status && data.length) {
      printOutputBuffer(data)

    }
  })
});

const send = (value) => {
  socket.write(`${JSON.stringify(value)}\n`)
}

const canvas = document.getElementById('canvas')
const getCanvasCoord = (pointNr, amplitude, totalPoints) => (
  [
    pointNr / totalPoints * canvas.clientWidth,
    canvas.clientHeight * (1 + amplitude / 10) / 2,
  ]
)

const ctx = canvas.getContext('2d');
const printOutputBuffer = data => {
  // Erase any previous line
  ctx.clearRect(0, 0, canvas.clientWidth, canvas.clientHeight)

  // Draw the baseline
  ctx.beginPath()
  ctx.lineWidth = 0.5
  ctx.strokeStyle = '#92c9dd'
  ctx.moveTo(0, canvas.clientHeight / 2)
  ctx.lineTo(canvas.clientWidth, canvas.clientHeight / 2);
  ctx.stroke()

  // Draw the rest
  ctx.beginPath()
  ctx.lineWidth = 1;
  ctx.strokeStyle = "#333";
  ctx.moveTo(...getCanvasCoord(0, data[0], data.length))

  data.slice(1).forEach((amplitude, pointNr) =>
    ctx.lineTo(...getCanvasCoord(pointNr, amplitude, data.length))
  )

  ctx.stroke()

  requestAnimationFrame(requestOutputData)

}

const requestOutputData = () => {
  send({ RootDataMessage: { target: 'OutputBuffer' } })
}

requestAnimationFrame(requestOutputData)

if (typeof window !== undefined) {
  document.getElementById('rangefreq').addEventListener('input', (e) => {
    send({ InstrumentMessage: { parameter: 'Frequency', value: parseInt(e.target.value) / 100 } })
  })
  document.getElementById('rangeq').addEventListener('input', (e) => {
    send({ InstrumentMessage: { parameter: 'Q', value: parseInt(e.target.value) / 100 } })
  })
}