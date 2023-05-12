const DEFAULT_CONNECT_TIMEOUT = 1000;
const DEFAULT_SEND_TIMEOUT = 1000;

export interface Transport {
  send: (msg: string) => Promise<void>
  onResponse: (handler: ResponseHandler) => () => void;
  onOpen: (handler: () => void) => () => void;
  destroy: () => void
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

export class EventEmitter<T> {
  map: Map<Symbol, (value: T) => void>
  constructor() {
    this.map = new Map();
  }

  addEventListener(listener: (value: T) => void) {
    let sym = Symbol();
    this.map.set(sym, listener)
    return () => {
      this.map.delete(sym)
    }
  }

  emit(value: T) {
    this.map.forEach(listener => listener(value))
  }
}


export class WebsocketTransport implements Transport {
  socket: WebSocket;
  config: Config;
  queue: Map<Symbol, { msg: string, resolve: (value: undefined) => void }>;
  events: {
    msg: EventEmitter<string>
    open: EventEmitter<undefined>
  }
  destroyed: boolean

  constructor(addr: string, config?: Partial<Config>) {
    this.destroyed = false;
    this.config = {
      connect_timeout: config?.connect_timeout || DEFAULT_CONNECT_TIMEOUT,
      send_timeout: config?.send_timeout || DEFAULT_SEND_TIMEOUT
    }
    this.queue = new Map();
    this.events = {
      msg: new EventEmitter(),
      open: new EventEmitter(),
    };
    this.connect(addr)
  }

  connect(addr: string) {
    this.socket = new WebSocket(addr)

    this.socket.onclose = () => {
      console.debug(`[zod-rpc] Disconnected from ${this.socket.url}`)
      setTimeout(() => {
        if (!this.destroyed) {
          this.connect(addr)
        }
      }, this.config.connect_timeout)
    };

    this.socket.onopen = () => {
      console.debug(`[zod-rpc] Connected to ${this.socket.url}`)
      this.events.open.emit(undefined)
      this.processDefered(msg => this.socket.send(msg))
    };

    this.socket.onmessage = evt => this.events.msg.emit(evt.data);
  }

  async send(msg: string): Promise<void> {
    if (this.socket.readyState == WebSocket.OPEN) {
      this.socket.send(msg)
    } else {
      await this.defer(msg);
    }
  }

  defer(msg: string): Promise<undefined> {
    return new Promise((resolve, reject) => {
      let sym = Symbol();
      this.queue.set(sym, { msg, resolve });
      setTimeout(() => {
        if (this.queue.delete(sym)) {
          reject(TimeoutError)
        }
      }, this.config.send_timeout)
    })
  }

  processDefered(next: (msg: string) => void) {
    this.queue.forEach(({ msg, resolve }) => {
      next(msg);
      resolve(undefined)
    })

    this.queue.clear();
  }


  onResponse(handler: ResponseHandler): () => void {
    return this.events.msg.addEventListener(handler);
  }

  onOpen(handler: () => void): () => void {
    return this.events.open.addEventListener(handler);
  }

  destroy() {
    this.destroyed = true;
    this.socket.close();
  }
}
