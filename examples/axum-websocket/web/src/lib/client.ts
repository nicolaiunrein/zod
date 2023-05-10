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
    let req_id = 0n;
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
      } else if ("cancel" in res) {
        const id = BigInt(res.stream.id);
        pending.delete(id);
      }
    };

    socket.onclose = onDisconnect;

    socket.onopen = () => {
      resolve({
        async call(ns, method, args) {
          req_id += 1n;
          let id = req_id;

          return new Promise((resolve, reject) => {
            let request = stringify_request({
              req_id: id,
              ns,
              method,
              args: [...args],
            });

            pending.set(id, { resolve, reject });
            socket.send(request);
          });
        },

        get_stream(ns, method, args) {
          req_id += 1n;
          let id = req_id;
          let request = stringify_request({
            req_id,
            ns,
            method,
            args: [...args],
          });

          let store = {
            subscribe(
              next: (
                value: { data: unknown } | { err: unknown } | { loading: true }
              ) => void
            ) {
              next({ loading: true });

              pending.set(id, {
                resolve: (data) => {
                  next({ data });
                },
                reject: (err) => {
                  next({ err });
                },
              });
              socket.send(request);
              return () => {
                pending.delete(id);
              };
            },
            close() {},
          };

          socket.send(request);

          return store;
        },
      });
    };
  });
};
