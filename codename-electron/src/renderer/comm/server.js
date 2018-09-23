import net from 'net'
import readline from 'readline'

export default class Server {
  constructor(port) {
    this.socket = new net.Socket()
    this.socket.setEncoding('utf8')
    this.socket.connect(port)
    const rl = readline.createInterface(this.socket, this.socket)
    rl.on('line', this.onLine.bind(this))

    // Keep track of callbacks
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