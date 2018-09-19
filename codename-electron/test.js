const net = require('net')

const socket = new net.Socket()
socket.setEncoding('utf8')
socket.connect(6142)
socket.on('error', err => console.error(err))
socket.on('data', data => {
  console.log(data);
});

const send = () => {
  socket.write(`Hello there\n`);
}

if (typeof window !== undefined) {
  document.getElementById('test').addEventListener('click', async () => {
    send()
  })
}