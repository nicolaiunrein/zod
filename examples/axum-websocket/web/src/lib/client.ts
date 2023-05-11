import { Rs } from "./api";

interface ExecPayload {
  id: bigint;
  ns: string;
  method: string;
  args: any[];
}

interface ClientConfig {
  addr: string,
  onDisconnect: () => void,
}

interface Resolver { resolve: (data: any) => void; reject: (error: any) => void }

function stringify_request(payload: ExecPayload): string {
  return JSON.stringify({ exec: payload }, (_, v) =>
    typeof v == "bigint" ? v.toString() : v
  );
}

export const connect = async ({ addr, onDisconnect }: ClientConfig) => {
  return new Promise<Rs.Client>((resolve) => {
    let next_req_id = 0n;
    let pending = new Map<bigint, Resolver>();


    const onMessage = (event: MessageEvent) => {
      let res = JSON.parse(event.data);
      if ("error" in res) {
        const id = BigInt(res.error.id);
        const promise = pending.get(id);
        const data = res.error.data;

        if (promise !== undefined) {
          promise.reject(data);
          pending.delete(id);
        }
      } else if ("method" in res) {
        const id = BigInt(res.method.id);
        const promise = pending.get(id);
        const data = res.method.data;

        if (promise !== undefined) {
          promise.resolve(data);
          pending.delete(id);
        }
      } else if ("stream" in res) {
        const id = BigInt(res.stream.id);
        const promise = pending.get(id);
        const data = res.stream.data;

        if (promise !== undefined) {
          promise.resolve(data);
        }
      } else {
        throw "Unexpected Response"
      }
    };


    const onOpen = () => {
      resolve({
        async call(ns, method, args) {
          next_req_id += 1n;
          let id = next_req_id;

          return new Promise((resolve, reject) => {
            let request = stringify_request({
              id,
              ns,
              method,
              args: [...args],
            });

            pending.set(id, { resolve, reject });
            socket.send(request);
          });
        },

        get_stream(ns, method, args) {
          next_req_id += 1n;
          let id = next_req_id;
          let request = stringify_request({
            id,
            ns,
            method,
            args,
          });

          let subscribers = new Map<Symbol, (evt: Rs.StreamEvent<unknown>) => void>;

          pending.set(id, {
            resolve: (data) => {
              subscribers.forEach(sub => {
                sub({ data });
              })
            },
            reject: (error) => {
              subscribers.forEach(sub => {
                sub({ error });
              })
            },
          });

          let store = {
            subscribe(
              next: (
                value: Rs.StreamEvent<unknown>
              ) => void
            ) {
              next({ loading: true });

              let sym = Symbol();

              subscribers.set(sym, next);

              return () => {
                subscribers.delete(sym);
                if (subscribers.size == 0) {
                  let request = JSON.stringify({ cancelStream: { id: id.toString() } });
                  socket.send(request);
                  pending.delete(id);
                }
              };
            },
          };

          socket.send(request);

          return store;
        },
      });
    };

    let socket = new WebSocket(addr);
    socket.onmessage = onMessage;
    socket.onclose = onDisconnect;
    socket.onopen = onOpen;
  });
};
