import { Rs } from "./api";
import type { Transport } from "./transport";

const DEFAULT_TIMEOUT = 1000;

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

interface Config {
  timeout: number
}


export class Client implements Rs.Client {
  transport: Transport;
  next_id: IdProvider;
  pending_responses: Map<bigint, (response: Rs.Response) => void>
  config: Config;

  constructor(transport: Transport, config: Partial<Config>) {
    this.transport = transport
    this.transport.onResponse(msg => {
      let res = JSON.parse(msg);
      let parsed_response = Rs.Response.parse(res);
      this.pending_responses.forEach(handler => handler(parsed_response))
    })
    this.next_id = new IdProvider();
    this.pending_responses = new Map();
    this.config = {
      timeout: config.timeout || DEFAULT_TIMEOUT
    }
  }

  async call(ns: string, method: string, args: any[]): Promise<Rs.Response> {
    let id = this.next_id.get();
    let msg = stringify_request({ id, ns, method, args })

    let res = new Promise<Rs.Response>((resolve, reject) => {

      this.pending_responses.set(id, (response: Rs.Response) => {
        resolve(response)
      })

      setTimeout(() => {
        if (this.pending_responses.delete(id)) {
          reject("Timeout")
        }
      }, this.config.timeout)
    })

    this.transport.send(msg);
    return await res;
  }



  get_stream(ns: string, method: string, args: unknown[]): Rs.Stream<Rs.StreamResponse> {

    let id = this.next_id.get();
    let msg = stringify_request({ id, ns, method, args })
    let subscribers = new Map<Symbol, (value: Rs.StreamEvent<unknown>) => void>

    this.pending_responses.set(id, data => {
      subscribers.forEach(subscriber => {
        subscriber({ data })
      });
    })

    this.transport.send(msg);


    return {
      subscribe(next) {
        let sym = Symbol();
        this.subscribers.set(sym, next);
        return () => {
          this.subscribers.delete(sym)
        }
      },
    }


  }

}
