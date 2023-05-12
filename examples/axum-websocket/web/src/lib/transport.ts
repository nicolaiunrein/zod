const DEFAULT_CONNECT_TIMEOUT = 2000;
const DEFAULT_RECONNECT_INTERVAL = 1000;
const DEFAULT_SEND_TIMEOUT = 1000;

type ConnectionState = "open" | "close";

export interface Transport {
  send: (msg: string) => Promise<void>
  onResponse: (handler: ResponseHandler) => () => void;
  onStateChange: (handler: (state: ConnectionState) => void) => () => void;
  destroy: () => void
}

class SendTimeoutError extends Error {
  name: string;
  message: string;

  constructor() {
    super()
    this.name = "SendTimeoutError"
    this.message = "Timeout sending request";
  }
}

type ResponseHandler = (msg: string) => void;

interface Config {
  connect_timeout: number,
  send_timeout: number
  reconnect_interval: number
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
    close: EventEmitter<undefined>
  }
  destroyed: boolean

  constructor(addr: string, config?: Partial<Config>) {
    this.destroyed = false;
    this.config = {
      connect_timeout: config?.connect_timeout || DEFAULT_CONNECT_TIMEOUT,
      reconnect_interval: config?.reconnect_interval || DEFAULT_RECONNECT_INTERVAL,
      send_timeout: config?.send_timeout || DEFAULT_SEND_TIMEOUT
    }
    this.queue = new Map();
    this.events = {
      msg: new EventEmitter(),
      open: new EventEmitter(),
      close: new EventEmitter(),
    };
    this.connect(addr)
  }

  connect(addr: string) {
    console.log("[zod-rpc] Connecting...");
    this.socket = new WebSocket(addr)
    setTimeout(() => {
      if (this.socket.readyState == WebSocket.CONNECTING) {
        this.socket.close();
      }
    }, this.config.connect_timeout);

    this.socket.onclose = () => {
      console.debug(`[zod-rpc] Disconnected from ${this.socket.url}`)
      this.events.close.emit(undefined)
      setTimeout(() => {
        if (!this.destroyed) {
          this.connect(addr)
        }
      }, this.config.reconnect_interval)
    };

    if (this.socket.readyState == WebSocket.OPEN) {
      console.debug(`[zod-rpc] Connected to ${this.socket.url}`)
      this.events.open.emit(undefined)
      this.processDefered(msg => this.socket.send(msg))
    } else {
      this.socket.onopen = () => {
        console.debug(`[zod-rpc] Connected to ${this.socket.url}`)
        this.events.open.emit(undefined)
        this.processDefered(msg => this.socket.send(msg))
      };
    }

    this.socket.onmessage = evt => this.events.msg.emit(evt.data);
  }

  async send(msg: string): Promise<void> {
    if (this.socket.readyState == WebSocket.OPEN) {
      this.socket.send(msg)
    } else {
      try {
        await this.defer(msg);

      } catch (e) {
        throw e;
      }
    }
  }

  defer(msg: string): Promise<undefined> {
    return new Promise((resolve, reject) => {
      let sym = Symbol();
      this.queue.set(sym, { msg, resolve });

      setTimeout(() => {
        if (this.queue.delete(sym)) {
          reject(new SendTimeoutError())
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

  onStateChange(handler: (state: ConnectionState) => void): () => void {
    const destroy_open = this.events.open.addEventListener(() => handler("open"));
    const destroy_close = this.events.close.addEventListener(() => handler("close"));
    return () => {
      destroy_open();
      destroy_close();
    }
  }


  destroy() {
    this.destroyed = true;
    this.socket.close();
  }
}
