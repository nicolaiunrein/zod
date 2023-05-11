const DEFAULT_CONNECT_TIMEOUT = 1000;
const DEFAULT_SEND_TIMEOUT = 1000;

export interface Transport {
  send: (msg: string) => Promise<void>
  onResponse: (handler: ResponseHandler) => void;
}

class TimeoutError implements Error {
  name: "Timeout"
  message: "Timeout sending request";

}

type ResponseHandler = (msg: string) => void;

interface Config {
  connect_timeout: number,
  send_timeout: number
}

export class WebsocketTransport implements Transport {
  socket: WebSocket;
  pending: Map<Symbol, string>;
  config: Config;

  constructor(addr: string, config: Partial<Config>) {
    this.connect(addr)
    this.pending = new Map();
    this.config = {
      connect_timeout: config.connect_timeout || DEFAULT_CONNECT_TIMEOUT,
      send_timeout: config.send_timeout || DEFAULT_SEND_TIMEOUT
    }
  }

  connect(addr: string) {
    this.socket = new WebSocket(addr)

    this.socket.addEventListener("close", () => {
      setTimeout(() => {
        this.connect(addr)
      }, this.config.connect_timeout)
    })
  }

  send(msg: string): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        if (this.socket.readyState == this.socket.OPEN) {
          this.socket.send(msg)
          resolve
        } else {
          let sym = Symbol();
          this.pending.set(sym, msg);
          setTimeout(() => {
            if (this.pending.delete(sym)) {
              reject(TimeoutError)
            }
          }, this.config.send_timeout)
        }
      } catch (err) {
        reject(err)
      }
    })
  }

  onResponse(handler: ResponseHandler) {
    this.socket.addEventListener("message", msg => handler(msg.data))
  }
}
