import * as z from "zod";

const WS_ADDR = "ws://localhost:8000/ws";

const reopenTimeouts = [100, 200, 1000, 3000];

function websocketStore(url: string) {
  let initialValue: unknown = undefined;
  let socket: WebSocket | undefined;
  let openPromise: Promise<undefined> | undefined;
  let reopenTimeoutHandler: any;
  let reopenCount = 0;

  const subscriptions = new Set<([id, data]: [bigint, unknown]) => void>();

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
      const res = JSON.parse(event.data);
      console.trace({ response: res });

      // TODO
      if ("method" in res) {
        initialValue = [res.method.id, res.method.data];
        subscriptions.forEach((subscription) =>
          subscription(initialValue as any)
        );
      } else if ("stream" in res) {
        initialValue = [res.stream.id, res.stream.data];
        subscriptions.forEach((subscription) =>
          subscription(initialValue as any)
        );
      }
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
      socket.send(value);
    }
  };

  return {
    send(value: string) {
      open_and_send(value);
    },
    subscribe(subscription: (value: [bigint, unknown]) => void) {
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
let req_id = 0n;

function execute({
  req_id,
  method,
  namespace,
  args,
}: {
  req_id: bigint;
  namespace: string;
  method: string;
  args: any[];
}): string {
  return JSON.stringify(
    { exec: { id: req_id, method, namespace, args } },
    (_, v) => (typeof v == "bigint" ? v.toString() : v)
  );
}

export function subscribe<T>(
  namespace: string,
  method: string,
  args: IArguments
): Store<T> {
  req_id += 1n;
  let id = req_id;
  let req = { req_id, namespace, method, args: [...args] };

  let store = {
    subscribe(cb: (value: T) => void) {
      return CONNECTION.subscribe(([res_id, data]) => {
        if (res_id == id) {
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
    req_id += 1n;
    let id = req_id;
    let request = { req_id, namespace, method, args: [...args] };
    let start = performance.now();

    unsubscribe = CONNECTION.subscribe(([res_id, data]: [bigint, any]) => {
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
export namespace Rs {
export type String = string;
export const String = z.string();
export type Usize = BigInt;
export const Usize = z.bigint().nonnegative().lt(2n ** 64n);

}
export namespace Pixera {
// @ts-ignore
export function debug_stream(): Store<Rs.String> {
    // phantom usage
    z.lazy(() => z.tuple([])).parse([...arguments]);
    return subscribe("Pixera", "debug_stream", arguments);
};

// @ts-ignore
export function hello_stream(num: Rs.Usize): Store<Rs.Usize> {
    // phantom usage
    num;
    z.lazy(() => z.tuple([Rs.Usize])).parse([...arguments]);
    return subscribe("Pixera", "hello_stream", arguments);
};

// @ts-ignore
export function y(): Store<Rs.String> {
    // phantom usage
    z.lazy(() => z.tuple([])).parse([...arguments]);
    return subscribe("Pixera", "y", arguments);
};


}
export namespace Watchout {
export interface Generic<T, V> { value: Rs.String, t: T, v: V }
export const Generic = (T: z.ZodTypeAny, V: z.ZodTypeAny) => z.object({ value: Rs.String, t: T, v: V });
export interface MyEntity { value: Rs.String }
export const MyEntity = z.lazy(() => z.object({ value: Rs.String }));
export interface User { value: Watchout.Generic<Rs.Usize, Rs.Usize> }
export const User = z.lazy(() => z.object({ value: Watchout.Generic(Rs.Usize, Rs.Usize) }));
// @ts-ignore
export async function hello(_s: Rs.String, _n: Rs.Usize): Promise<Rs.Usize> {
    // phantom usage
    _s;
    _n;
    z.lazy(() => z.tuple([Rs.String, Rs.Usize])).parse([...arguments]);
    return request("Watchout", "hello", arguments);
};

// @ts-ignore
export async function hello1(_s: Rs.String): Promise<Rs.Usize> {
    // phantom usage
    _s;
    z.lazy(() => z.tuple([Rs.String])).parse([...arguments]);
    return request("Watchout", "hello1", arguments);
};

// @ts-ignore
export function hello_stream(num: Rs.Usize): Store<Rs.Usize> {
    // phantom usage
    num;
    z.lazy(() => z.tuple([Rs.Usize])).parse([...arguments]);
    return subscribe("Watchout", "hello_stream", arguments);
};

// @ts-ignore
export async function hello_user(_user: Watchout.User, _n: Rs.Usize): Promise<Rs.Usize> {
    // phantom usage
    _user;
    _n;
    z.lazy(() => z.tuple([Watchout.User, Rs.Usize])).parse([...arguments]);
    return request("Watchout", "hello_user", arguments);
};

// @ts-ignore
export async function nested(_value: Watchout.MyEntity): Promise<Rs.Usize> {
    // phantom usage
    _value;
    z.lazy(() => z.tuple([Watchout.MyEntity])).parse([...arguments]);
    return request("Watchout", "nested", arguments);
};

// @ts-ignore
export async function test(_user: Watchout.User, _n: Rs.Usize): Promise<Rs.Usize> {
    // phantom usage
    _user;
    _n;
    z.lazy(() => z.tuple([Watchout.User, Rs.Usize])).parse([...arguments]);
    return request("Watchout", "test", arguments);
};


}

