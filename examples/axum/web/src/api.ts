import { z } from "zod";

export namespace Rs {
  export type String = string;

  export const String = z.string();

  export type Usize = BigInt;

  export const Usize = z.coerce
    .bigint()
    .nonnegative()
    .lt(2n ** 64n);

  export interface Client {
    get_stream(ns: string, method: string, args: unknown[]): Stream<unknown>;
    call(ns: string, method: string, args: unknown[]): Promise<unknown>;
  }
  export interface Stream<T> {
    subscribe(next: (value: T) => void): () => void;
  }
}

export namespace Pixera {
  export function init(client: Rs.Client) {
    return {
      // @ts-ignore
      debug_stream(): Rs.Stream<Rs.String> {
        z.lazy(() => z.tuple([])).parse([]);
        return {
          subscribe(cb) {
            return client
              .get_stream("Pixera", "debug_stream", [])
              .subscribe((val) => {
                cb(Rs.String.parse(val));
              });
          },
        };
      },

      // @ts-ignore
      hello_stream(num: Rs.Usize): Rs.Stream<Rs.Usize> {
        z.lazy(() => z.tuple([Rs.Usize])).parse([num]);
        return {
          subscribe(cb) {
            return client
              .get_stream("Pixera", "hello_stream", [num])
              .subscribe((val) => {
                cb(Rs.Usize.parse(val));
              });
          },
        };
      },

      // @ts-ignore
      y(): Rs.Stream<Rs.String> {
        z.lazy(() => z.tuple([])).parse([]);
        return {
          subscribe(cb) {
            return client.get_stream("Pixera", "y", []).subscribe((val) => {
              cb(Rs.String.parse(val));
            });
          },
        };
      },
    };
  }
}

export namespace Watchout {
  export interface Generic<T, V> {
    value: Rs.String;
    t: T;
    v: V;
  }

  export const Generic = (T: z.ZodTypeAny, V: z.ZodTypeAny) =>
    z.object({ value: Rs.String, t: T, v: V });

  export interface MyEntity {
    value: Rs.String;
  }

  export const MyEntity = z.lazy(() => z.object({ value: Rs.String }));

  export interface User {
    value: Watchout.Generic<Rs.Usize, Rs.Usize>;
  }

  export const User = z.lazy(() =>
    z.object({ value: Watchout.Generic(Rs.Usize, Rs.Usize) })
  );

  export function init(client: Rs.Client) {
    return {
      // @ts-ignore
      async hello(_s: Rs.String, _n: Rs.Usize): Promise<Rs.Usize> {
        z.lazy(() => z.tuple([Rs.String, Rs.Usize])).parse([_s, _n]);
        return Rs.Usize.parse(await client.call("Watchout", "hello", [_s, _n]));
      },

      // @ts-ignore
      async hello1(_s: Rs.String): Promise<Rs.Usize> {
        z.lazy(() => z.tuple([Rs.String])).parse([_s]);
        return Rs.Usize.parse(await client.call("Watchout", "hello1", [_s]));
      },

      // @ts-ignore
      hello_stream(num: Rs.Usize): Rs.Stream<Rs.Usize> {
        z.lazy(() => z.tuple([Rs.Usize])).parse([num]);
        return {
          subscribe(cb) {
            return client
              .get_stream("Watchout", "hello_stream", [num])
              .subscribe((val) => {
                cb(Rs.Usize.parse(val));
              });
          },
        };
      },

      // @ts-ignore
      async hello_user(_user: Watchout.User, _n: Rs.Usize): Promise<Rs.Usize> {
        z.lazy(() => z.tuple([Watchout.User, Rs.Usize])).parse([_user, _n]);
        return Rs.Usize.parse(
          await client.call("Watchout", "hello_user", [_user, _n])
        );
      },

      // @ts-ignore
      async nested(_value: Watchout.MyEntity): Promise<Rs.Usize> {
        z.lazy(() => z.tuple([Watchout.MyEntity])).parse([_value]);
        return Rs.Usize.parse(
          await client.call("Watchout", "nested", [_value])
        );
      },

      // @ts-ignore
      async newtype(value: Watchout.MyEntity): Promise<Rs.String> {
        z.lazy(() => z.tuple([Watchout.MyEntity])).parse([value]);
        return Rs.String.parse(
          await client.call("Watchout", "newtype", [value])
        );
      },

      // @ts-ignore
      async test(_user: Watchout.User, _n: Rs.Usize): Promise<Rs.Usize> {
        z.lazy(() => z.tuple([Watchout.User, Rs.Usize])).parse([_user, _n]);
        return Rs.Usize.parse(
          await client.call("Watchout", "test", [_user, _n])
        );
      },
    };
  }
}
