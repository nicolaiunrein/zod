import { Rs } from "./api";
import type { Transport } from "./transport";


interface ExecPayload {
  id: bigint;
  ns: string;
  method: string;
  args: any[];
}

function stringify_request(payload: ExecPayload): string {
  return JSON.stringify({ exec: payload }, (_, v) =>
    typeof v == "bigint" ? v.toString() : v
  );
}


class IdProvider {
  current = 0n;

  get() {
    this.current += 1n;
    return this.current;
  }
}

export class ReceiveError extends Error {
  name: string;
  message: string;
  constructor(ctx: { ns: string, method: string, args: any[] }) {
    super()
    this.name = "ReceiveError";
    this.message = `Connection closed while receiving response for \`${ctx.ns}.${ctx.method}(${ctx.args.join(", ")})\``;
  }
}

export class Client implements Rs.Client {
  transport: Transport;
  next_id: IdProvider;
  listeners: Map<bigint, (response: Rs.Response) => void>

  constructor(transport: Transport) {
    this.listeners = new Map();
    this.transport = transport
    this.transport.onResponse(msg => {
      let res = JSON.parse(msg);
      let parsed_response = Rs.Response.parse(res);
      let id = "method" in parsed_response ? parsed_response.method.id : "stream" in parsed_response ? parsed_response.stream.id : parsed_response.error.id;

      for (let [res_id, handler] of this.listeners) {
        if (res_id == id) {
          handler(parsed_response)
        }
      }
    });

    this.next_id = new IdProvider();
  }

  destroy() {
    this.transport.destroy()
  }


  async call(ns: string, method: string, args: any[]): Promise<unknown> {
    let id = this.next_id.get();
    let msg = stringify_request({ id, ns, method, args })

    await this.transport.send(msg);

    return new Promise<unknown>((resolve, reject) => {
      let destroy = this.transport.onStateChange(state => {
        if (state == "close") {
          this.listeners.delete(id)
          reject(new ReceiveError({ method, ns, args }))
        }
      });

      this.listeners.set(id, (response: Rs.Response) => {
        destroy();
        if ("method" in response) {
          resolve(response.method.data)
        } else if ("error" in response) {
          reject(response.error.data)
        }
        this.listeners.delete(id)
      })
    })
  }



  get_stream(ns: string, method: string, args: unknown[]): Rs.Stream<unknown> {

    let id = this.next_id.get();
    let msg = stringify_request({ id, ns, method, args })
    let subscribers = new Map<Symbol, (value: Rs.StreamEvent<unknown>) => void>

    this.listeners.set(id, res => {
      if ("stream" in res) {
        subscribers.forEach(subscriber => {
          subscriber({ data: res.stream.data })
        });
      } else if ("error" in res) {
        subscribers.forEach(subscriber => {
          // TODO
          subscriber({ error: res.error.data as any })
        });
      }
    })

    return {
      subscribe: (next) => {
        next({ loading: true })
        let sym = Symbol();
        let destroy: (() => void) | undefined;

        subscribers.set(sym, next);
        if (subscribers.size == 1) {
          this.transport.send(msg).catch(error => next({ error }));
          destroy = this.transport.onStateChange((state) => {
            if (state == "open") {
              this.transport.send(msg).catch(error => next({ error }));
            } else if (state == "close") {
              next({ loading: true })
            }
          });
        }
        return () => {
          subscribers.delete(sym)
          if (subscribers.size == 0) {
            let request = JSON.stringify({ cancelStream: { id: id.toString() } });
            this.transport.send(request).catch((e) => console.error(`Error canceling stream ${e.message}`))
            if (destroy !== undefined) {
              destroy()
            }
          }
        }
      },
    }


  }

}
