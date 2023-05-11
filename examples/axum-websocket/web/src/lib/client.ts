import { Rs } from "./api";

function stringify_request({
  req_id,
  method,
  ns,
  args,
}: {
  req_id: bigint;
  ns: string;
  method: string;
  args: any[];
}): string {
  return JSON.stringify({ exec: { id: req_id, method, ns, args } }, (_, v) =>
    typeof v == "bigint" ? v.toString() : v
  );
}

export const connect = async (addr: string, onDisconnect: () => void) => {
  return new Promise<Rs.Client>((resolve) => {
    let next_req_id = 0n;
    let socket = new WebSocket(addr);

    let pending = new Map<
      bigint,
      { resolve: (data: any) => void; reject: (error: any) => void }
    >();

    socket.onmessage = (event) => {
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

    socket.onclose = onDisconnect;

    socket.onopen = () => {
      resolve({
        async call(ns, method, args) {
          next_req_id += 1n;
          let req_id = next_req_id;

          return new Promise((resolve, reject) => {
            let request = stringify_request({
              req_id,
              ns,
              method,
              args: [...args],
            });

            pending.set(req_id, { resolve, reject });
            socket.send(request);
          });
        },

        get_stream(ns, method, args) {
          console.log(`get_stream ${method}`);
          next_req_id += 1n;
          let req_id = next_req_id;
          let request = stringify_request({
            req_id,
            ns,
            method,
            args: [...args],
          });

          let subscribers = new Map<Symbol, (evt: Rs.StreamEvent<unknown>) => void>;

          pending.set(req_id, {
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
                  let request = JSON.stringify({ cancelStream: { id: req_id.toString() } });
                  socket.send(request);
                  pending.delete(req_id);
                }
              };
            },
          };

          socket.send(request);

          return store;
        },
      });
    };
  });
};
