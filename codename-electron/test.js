const net = require('net')

const socket = new net.Socket()
socket.setEncoding('utf8')
socket.connect(6142)
socket.on('error', err => console.error(err))
// socket.on('data', data => {
//   console.log(data);
// });

const send = (value) => {
  socket.write(`${JSON.stringify(value)}\n`)
}

if (typeof window !== undefined) {
  document.getElementById('rangefreq').addEventListener('input', (e) => {
    send({ parameter: 'Frequency', value: parseInt(e.target.value) / 100 })
  })
  document.getElementById('rangeq').addEventListener('input', (e) => {
    send({ parameter: 'Q', value: parseInt(e.target.value) / 100 })
  })
}