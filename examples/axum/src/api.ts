const WS_ADDR = "ws://localhost:8000/ws";

const reopenTimeouts = [100, 200, 1000, 3000];

function websocketStore(url: string) {
  let initialValue: unknown = undefined;
  let socket: WebSocket | undefined;
  let openPromise: Promise<undefined> | undefined;
  let reopenTimeoutHandler: any;
  let reopenCount = 0;

  const subscriptions = new Set<([id, data]: [number, unknown]) => void>();

  function reopenTimeout() {
    const n = reopenCount;
    reopenCount++;
    return reopenTimeouts[
      n >= reopenTimeouts.length - 1 ? reopenTimeouts.length - 1 : n
    ];
  }

  function close() {
    if (reopenTimeoutHandler) {
      clearTimeout(reopenTimeoutHandler);
    }

    if (socket) {
      socket.close();
      socket = undefined;
    }
  }

  function reopen() {
    close();
    if (subscriptions.size > 0) {
      reopenTimeoutHandler = setTimeout(() => open(), reopenTimeout());
    }
  }

  async function open(): Promise<undefined> {
    if (reopenTimeoutHandler) {
      clearTimeout(reopenTimeoutHandler);
      reopenTimeoutHandler = undefined;
    }

    // we are still in the opening phase
    if (openPromise) {
      return openPromise;
    }

    console.debug("creating a new socket");
    socket = new WebSocket(url);

    socket.onmessage = (event) => {
      initialValue = JSON.parse(event.data);
      console.debug(initialValue);
      subscriptions.forEach((subscription) =>
        subscription(initialValue as any)
      );
    };

    socket.onclose = (event: Event) => {
      if (subscriptions.size > 0) {
        console.warn(event);
      }
      reopen();
    };

    openPromise = new Promise((resolve, reject) => {
      if (!socket) {
        openPromise = undefined;
        return;
      }

      socket.onerror = (error) => {
        reject(error);
        openPromise = undefined;
      };
      socket.onopen = (_: Event) => {
        reopenCount = 0;
        resolve(undefined);
        openPromise = undefined;
      };
    });
    return openPromise;
  }

  const open_and_send = (value: string) => {
    if (!socket || socket.readyState !== WebSocket.OPEN)
      open().then(() => open_and_send(value));
    else {
      // console.log("sending request", value);
      socket.send(value);
    }
  };

  return {
    send(value: string) {
      open_and_send(value);
    },
    subscribe(subscription: (value: [number, unknown]) => void) {
      subscriptions.add(subscription);
      return () => {
        subscriptions.delete(subscription);
        if (subscriptions.size === 0) {
          close();
        }
      };
    },
  };
}

type Store<T> = {
  subscribe(subscriber: (value: T) => void): () => void;
  close(): void;
};

const CONNECTION = websocketStore(WS_ADDR);
let req_id = 0;

function execute({
  req_id,
  method,
  namespace,
  args,
}: {
  req_id: number;
  namespace: string;
  method: string;
  args: any[];
}): string {
  return JSON.stringify({ exec: { id: req_id, method, namespace, args } });
}

export function subscribe<T>(
  namespace: string,
  method: string,
  args: IArguments
): Store<T> {
  req_id += 1;
  let id = req_id;
  let req = { req_id, namespace, method, args: [...args] };

  let store = {
    subscribe(cb: (value: T) => void) {
      return CONNECTION.subscribe(([res_id, data]) => {
        if (res_id == id) {
          console.log("Stream Response", {
            req_id,
            request: req,
            response: data,
          });
          cb(data as T);
        }
      });
    },
    close() {},
  };

  CONNECTION.send(execute(req));

  return store;
}

export async function request<T>(
  namespace: string,
  method: string,
  args: IArguments
): Promise<T> {
  let unsubscribe: () => void | undefined;
  let promise = new Promise((resolve: (_: T) => void, _) => {
    req_id += 1;
    let id = req_id;
    let request = { req_id, namespace, method, args: [...args] };
    let start = performance.now();

    unsubscribe = CONNECTION.subscribe(([res_id, data]: [number, any]) => {
      if (res_id == id) {
        console.log("Exec Response", {
          req_id,
          request,
          response: data,
          exec_duration_ms: performance.now() - start,
        });
        resolve(data);
      }
    });

    CONNECTION.send(execute(request));
  });

  return promise
    .then((value: T) => {
      return value;
    })
    .finally(() => unsubscribe && unsubscribe());
}
export namespace Watchout {

                    // @ts-ignore
                    export function hello_stream(num: number): Promise<null> {
                    return request("Watchout", "hello_stream", arguments);
                };
                    // @ts-ignore
                    export function hello(s: string,num: number): Promise<number> {
                    return request("Watchout", "hello", arguments);
                };export interface MyEntity { value: Pixera.MyEntity2, };
};
