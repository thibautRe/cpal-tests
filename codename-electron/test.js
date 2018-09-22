const net = require('net')

const socket = new net.Socket()
socket.setEncoding('utf8')
socket.connect(6142)
socket.on('error', err => console.error(err))
socket.on('data', data => {
  console.log(data);
});

const send = (value) => {
  const json = JSON.stringify({ value })
  socket.write(`${json}\n`)
}

if (typeof window !== undefined) {
  document.getElementById('range').addEventListener('input', (e) => {
    send(parseInt(e.target.value))
  })
}