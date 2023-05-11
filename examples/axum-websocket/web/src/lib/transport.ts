const DEFAULT_CONNECT_TIMEOUT = 1000;
const DEFAULT_SEND_TIMEOUT = 1000;

export interface Transport {
  send: (msg: string) => Promise<void>
  onResponse: (handler: ResponseHandler) => void;
  onOpen: (handler: () => void) => void;
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
  pending: Map<Symbol, { msg: string, resolve: () => void }>;
  config: Config;
  onMessageHandlers: ResponseHandler[]
  onOpenHandlers: Array<() => void>

  constructor(addr: string, config?: Partial<Config>) {
    this.connect(addr)
    this.pending = new Map();
    this.onMessageHandlers = [];
    this.onOpenHandlers = [];
    this.config = {
      connect_timeout: config?.connect_timeout || DEFAULT_CONNECT_TIMEOUT,
      send_timeout: config?.send_timeout || DEFAULT_SEND_TIMEOUT
    }
  }

  connect(addr: string) {
    this.socket = new WebSocket(addr)

    this.socket.addEventListener("close", () => {
      console.log("Disconnected")
      setTimeout(() => {
        this.connect(addr)
      }, this.config.connect_timeout)
    })
    this.socket.addEventListener("open", () => {
      console.log("Connected")
      this.pending.forEach(({ msg, resolve }) => {
        this.socket.send(msg);
        resolve()

      })
      this.pending.clear()
      this.onOpenHandlers.forEach(handle => handle())
    })

    this.socket.addEventListener("message", evt => this.onMessageHandlers.forEach(handle => handle(evt.data)))
  }

  send(msg: string): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        if (this.socket.readyState == 1) {
          this.socket.send(msg)
          resolve()
        } else {
          let sym = Symbol();
          this.pending.set(sym, {
            msg,
            resolve,
          });
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
    this.onMessageHandlers.push(handler);
  }

  onOpen(handler: () => void) {
    this.onOpenHandlers.push(handler)

  }
}
