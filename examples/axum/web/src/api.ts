import * as z from "zod";

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

export namespace Rs {
  export const BTreeMap = (K: z.ZodTypeAny, V: z.ZodTypeAny) =>
    z.map(
      z.lazy(() => K),
      z.lazy(() => V)
    );
  export type BTreeMap<K, V> = Map<K, V>;

  export const BTreeSet = (T: z.ZodTypeAny) => z.set(z.lazy(() => T));
  export type BTreeSet<T> = Set<T>;

  export const Bool = z.boolean();
  export type Bool = boolean;

  export const Char = z.string().length(1);
  export type Char = string;

  export const F32 = z.number();
  export type F32 = number;

  export const F64 = z.number();
  export type F64 = number;

  export const HashMap = (K: z.ZodTypeAny, V: z.ZodTypeAny) =>
    z.map(
      z.lazy(() => K),
      z.lazy(() => V)
    );
  export type HashMap<K, V> = Map<K, V>;

  export const HashSet = (T: z.ZodTypeAny) => z.set(z.lazy(() => T));
  export type HashSet<T> = Set<T>;

  export const I128 = z
    .number()
    .finite()
    .int()
    .lte(170141183460469231731687303715884105727)
    .gte(-170141183460469231731687303715884105728);
  export type I128 = number;

  export const I16 = z.number().finite().int().lte(32767).gte(-32768);
  export type I16 = number;

  export const I32 = z.number().finite().int().lte(2147483647).gte(-2147483648);
  export type I32 = number;

  export const I64 = z
    .number()
    .finite()
    .int()
    .lte(9223372036854775807)
    .gte(-9223372036854775808);
  export type I64 = number;

  export const I8 = z.number().finite().int().lte(127).gte(-128);
  export type I8 = number;

  export const IpAddr = z.string().ip();
  export type IpAddr = string;

  export const Ipv4Addr = z.string().ip({ version: "v4" });
  export type Ipv4Addr = string;

  export const Ipv6Addr = z.string().ip({ version: "v6" });
  export type Ipv6Addr = string;

  export const Isize = z.number().finite().int();
  export type Isize = number;

  export const Option = (T: z.ZodTypeAny) => z.lazy(() => T.optional());
  export type Option<T> = T | undefined;

  export const Result = (T: z.ZodTypeAny, E: z.ZodTypeAny) =>
    z.union([
      z.object({ Ok: z.lazy(() => T) }),
      z.object({ Err: z.lazy(() => E) }),
    ]);
  export type Result<T, E> = { Ok: T } | { Err: E };

  export const String = z.string();
  export type String = string;

  export const Tuple1 = (T1: z.ZodTypeAny) => z.tuple([z.lazy(() => T1)]);
  export type Tuple1<T1> = [T1];

  export const Tuple10 = (
    T1: z.ZodTypeAny,
    T2: z.ZodTypeAny,
    T3: z.ZodTypeAny,
    T4: z.ZodTypeAny,
    T5: z.ZodTypeAny,
    T6: z.ZodTypeAny,
    T7: z.ZodTypeAny,
    T8: z.ZodTypeAny,
    T9: z.ZodTypeAny,
    T10: z.ZodTypeAny
  ) =>
    z.tuple([
      z.lazy(() => T1),
      z.lazy(() => T2),
      z.lazy(() => T3),
      z.lazy(() => T4),
      z.lazy(() => T5),
      z.lazy(() => T6),
      z.lazy(() => T7),
      z.lazy(() => T8),
      z.lazy(() => T9),
      z.lazy(() => T10),
    ]);
  export type Tuple10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> = [
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
    T10
  ];

  export const Tuple11 = (
    T1: z.ZodTypeAny,
    T2: z.ZodTypeAny,
    T3: z.ZodTypeAny,
    T4: z.ZodTypeAny,
    T5: z.ZodTypeAny,
    T6: z.ZodTypeAny,
    T7: z.ZodTypeAny,
    T8: z.ZodTypeAny,
    T9: z.ZodTypeAny,
    T10: z.ZodTypeAny,
    T11: z.ZodTypeAny
  ) =>
    z.tuple([
      z.lazy(() => T1),
      z.lazy(() => T2),
      z.lazy(() => T3),
      z.lazy(() => T4),
      z.lazy(() => T5),
      z.lazy(() => T6),
      z.lazy(() => T7),
      z.lazy(() => T8),
      z.lazy(() => T9),
      z.lazy(() => T10),
      z.lazy(() => T11),
    ]);
  export type Tuple11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> = [
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
    T10,
    T11
  ];

  export const Tuple12 = (
    T1: z.ZodTypeAny,
    T2: z.ZodTypeAny,
    T3: z.ZodTypeAny,
    T4: z.ZodTypeAny,
    T5: z.ZodTypeAny,
    T6: z.ZodTypeAny,
    T7: z.ZodTypeAny,
    T8: z.ZodTypeAny,
    T9: z.ZodTypeAny,
    T10: z.ZodTypeAny,
    T11: z.ZodTypeAny,
    T12: z.ZodTypeAny
  ) =>
    z.tuple([
      z.lazy(() => T1),
      z.lazy(() => T2),
      z.lazy(() => T3),
      z.lazy(() => T4),
      z.lazy(() => T5),
      z.lazy(() => T6),
      z.lazy(() => T7),
      z.lazy(() => T8),
      z.lazy(() => T9),
      z.lazy(() => T10),
      z.lazy(() => T11),
      z.lazy(() => T12),
    ]);
  export type Tuple12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> = [
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
    T10,
    T11,
    T12
  ];

  export const Tuple2 = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) =>
    z.tuple([z.lazy(() => T1), z.lazy(() => T2)]);
  export type Tuple2<T1, T2> = [T1, T2];

  export const Tuple3 = (
    T1: z.ZodTypeAny,
    T2: z.ZodTypeAny,
    T3: z.ZodTypeAny
  ) => z.tuple([z.lazy(() => T1), z.lazy(() => T2), z.lazy(() => T3)]);
  export type Tuple3<T1, T2, T3> = [T1, T2, T3];

  export const Tuple4 = (
    T1: z.ZodTypeAny,
    T2: z.ZodTypeAny,
    T3: z.ZodTypeAny,
    T4: z.ZodTypeAny
  ) =>
    z.tuple([
      z.lazy(() => T1),
      z.lazy(() => T2),
      z.lazy(() => T3),
      z.lazy(() => T4),
    ]);
  export type Tuple4<T1, T2, T3, T4> = [T1, T2, T3, T4];

  export const Tuple5 = (
    T1: z.ZodTypeAny,
    T2: z.ZodTypeAny,
    T3: z.ZodTypeAny,
    T4: z.ZodTypeAny,
    T5: z.ZodTypeAny
  ) =>
    z.tuple([
      z.lazy(() => T1),
      z.lazy(() => T2),
      z.lazy(() => T3),
      z.lazy(() => T4),
      z.lazy(() => T5),
    ]);
  export type Tuple5<T1, T2, T3, T4, T5> = [T1, T2, T3, T4, T5];

  export const Tuple6 = (
    T1: z.ZodTypeAny,
    T2: z.ZodTypeAny,
    T3: z.ZodTypeAny,
    T4: z.ZodTypeAny,
    T5: z.ZodTypeAny,
    T6: z.ZodTypeAny
  ) =>
    z.tuple([
      z.lazy(() => T1),
      z.lazy(() => T2),
      z.lazy(() => T3),
      z.lazy(() => T4),
      z.lazy(() => T5),
      z.lazy(() => T6),
    ]);
  export type Tuple6<T1, T2, T3, T4, T5, T6> = [T1, T2, T3, T4, T5, T6];

  export const Tuple7 = (
    T1: z.ZodTypeAny,
    T2: z.ZodTypeAny,
    T3: z.ZodTypeAny,
    T4: z.ZodTypeAny,
    T5: z.ZodTypeAny,
    T6: z.ZodTypeAny,
    T7: z.ZodTypeAny
  ) =>
    z.tuple([
      z.lazy(() => T1),
      z.lazy(() => T2),
      z.lazy(() => T3),
      z.lazy(() => T4),
      z.lazy(() => T5),
      z.lazy(() => T6),
      z.lazy(() => T7),
    ]);
  export type Tuple7<T1, T2, T3, T4, T5, T6, T7> = [T1, T2, T3, T4, T5, T6, T7];

  export const Tuple8 = (
    T1: z.ZodTypeAny,
    T2: z.ZodTypeAny,
    T3: z.ZodTypeAny,
    T4: z.ZodTypeAny,
    T5: z.ZodTypeAny,
    T6: z.ZodTypeAny,
    T7: z.ZodTypeAny,
    T8: z.ZodTypeAny
  ) =>
    z.tuple([
      z.lazy(() => T1),
      z.lazy(() => T2),
      z.lazy(() => T3),
      z.lazy(() => T4),
      z.lazy(() => T5),
      z.lazy(() => T6),
      z.lazy(() => T7),
      z.lazy(() => T8),
    ]);
  export type Tuple8<T1, T2, T3, T4, T5, T6, T7, T8> = [
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8
  ];

  export const Tuple9 = (
    T1: z.ZodTypeAny,
    T2: z.ZodTypeAny,
    T3: z.ZodTypeAny,
    T4: z.ZodTypeAny,
    T5: z.ZodTypeAny,
    T6: z.ZodTypeAny,
    T7: z.ZodTypeAny,
    T8: z.ZodTypeAny,
    T9: z.ZodTypeAny
  ) =>
    z.tuple([
      z.lazy(() => T1),
      z.lazy(() => T2),
      z.lazy(() => T3),
      z.lazy(() => T4),
      z.lazy(() => T5),
      z.lazy(() => T6),
      z.lazy(() => T7),
      z.lazy(() => T8),
      z.lazy(() => T9),
    ]);
  export type Tuple9<T1, T2, T3, T4, T5, T6, T7, T8, T9> = [
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9
  ];

  export const U128 = z
    .number()
    .finite()
    .int()
    .nonnegative()
    .lte(340282366920938463463374607431768211455);
  export type U128 = number;

  export const U16 = z.number().finite().int().nonnegative().lte(65535);
  export type U16 = number;

  export const U32 = z.number().finite().int().nonnegative().lte(4294967295);
  export type U32 = number;

  export const U64 = z
    .number()
    .finite()
    .int()
    .nonnegative()
    .lte(18446744073709551615);
  export type U64 = number;

  export const U8 = z.number().finite().int().nonnegative().lte(255);
  export type U8 = number;

  export const Unit = z.null();
  export type Unit = null;

  export const Usize = z.number().finite().int().nonnegative();
  export type Usize = number;
}
export namespace Watchout {
  export const Generic = (T: z.ZodTypeAny, V: z.ZodTypeAny) =>
    z.lazy(() => z.object({ value: Rs.String, t: T, v: V }));
  export interface Generic<T, V> {
    value: Rs.String;
    t: T;
    v: V;
  }

  export const MyEntity = z.lazy(() => z.object({ value: Pixera.MyEntity2 }));
  export interface MyEntity {
    value: Pixera.MyEntity2;
  }

  export const MyEntity3 = z.lazy(() => z.object({ value: Pixera.MyEntity2 }));
  export interface MyEntity3 {
    value: Pixera.MyEntity2;
  }

  export const T = z.lazy(() => Rs.Usize);
  export type T = Rs.Usize;

  export const User = z.lazy(() =>
    z.object({ value: Watchout.Generic(Rs.String, Rs.Usize) })
  );
  export interface User {
    value: Watchout.Generic<Rs.String, Rs.Usize>;
  }

  // @ts-ignore
  export async function hello(_s: Rs.String, _n: Rs.Usize): Promise<Usize> {
    // phantom usage
    Rs.String;
    Rs.Usize;

    z.lazy(() => z.tuple([Rs.String, Rs.Usize])).parse([...arguments]);
    return request("Watchout", "hello", arguments);
  }

  // @ts-ignore
  export async function hello1(_s: Rs.String): Promise<Usize> {
    // phantom usage
    Rs.String;

    z.lazy(() => z.tuple([Rs.String])).parse([...arguments]);
    return request("Watchout", "hello1", arguments);
  }

  // @ts-ignore
  export function hello_stream(num: Rs.Usize): Store<Usize> {
    // phantom usage
    Rs.Usize;

    z.lazy(() => z.tuple([Rs.Usize])).parse([...arguments]);
    return subscribe("Watchout", "hello_stream", arguments);
  }

  // @ts-ignore
  export async function nested(_value: Watchout.MyEntity): Promise<Usize> {
    // phantom usage
    Watchout.MyEntity;

    z.lazy(() => z.tuple([Watchout.MyEntity])).parse([...arguments]);
    return request("Watchout", "nested", arguments);
  }
}
export namespace Pixera {
  export const MyEntity2 = z.lazy(() => z.object({ value: Rs.Usize }));
  export interface MyEntity2 {
    value: Rs.Usize;
  }

  // @ts-ignore
  export function x(): Store<String> {
    z.lazy(() => z.tuple([])).parse([...arguments]);
    return subscribe("Pixera", "x", arguments);
  }

  // @ts-ignore
  export function y(): Store<String> {
    z.lazy(() => z.tuple([])).parse([...arguments]);
    return subscribe("Pixera", "y", arguments);
  }
}
