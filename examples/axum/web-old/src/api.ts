import * as z from "zod";

export namespace Rs {
  export type String = string;
  export type Usize = number;
  export const String = z.string();
  export const Usize = z.number();

  export type Store<T> = {
    subscribe(subscriber: (value: T) => void): () => void;
    close(): void;
  };

  export interface Client {
    request: (ns: string, name: string, args: IArguments) => any;
    subscribe: (ns: string, name: string, args: IArguments) => Store<any>;
  }
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
    z.object({ value: Watchout.Generic(Rs.Usize, Rs.Usize) })
  );
  export interface User {
    value: Watchout.Generic<Rs.Usize, Rs.Usize>;
  }

  // @ts-ignore
  export async function hello(
    client: Rs.Client,
    _s: Rs.String,
    _n: Rs.Usize
  ): Promise<Rs.Usize> {
    // phantom usage
    Rs.String;
    Rs.Usize;

    z.lazy(() => z.tuple([Rs.String, Rs.Usize])).parse([...arguments]);
    return client.request("Watchout", "hello", arguments);
  }

  // @ts-ignore
  export async function hello1(client: Client, _s: Rs.String): Promise<Usize> {
    // phantom usage
    Rs.String;

    z.lazy(() => z.tuple([Rs.String])).parse([...arguments]);
    return client.request("Watchout", "hello1", arguments);
  }

  // @ts-ignore
  export function hello_stream(
    client: Rs.Client,
    _num: Rs.Usize
  ): Rs.Store<Rs.Usize> {
    // phantom usage
    Rs.Usize;

    z.lazy(() => z.tuple([Rs.Usize])).parse([...arguments]);
    return client.subscribe("Watchout", "hello_stream", arguments);
  }

  // @ts-ignore
  export async function hello_user(
    client: Rs.Client,
    _user: Watchout.User,
    _n: Rs.Usize
  ): Promise<Rs.Usize> {
    // phantom usage
    Watchout.User;
    Rs.Usize;

    z.lazy(() => z.tuple([Watchout.User, Rs.Usize])).parse([...arguments]);
    return client.request("Watchout", "hello_user", arguments);
  }

  // @ts-ignore
  export async function nested(
    client: Rs.Client,
    _value: Watchout.MyEntity
  ): Promise<Rs.Usize> {
    // phantom usage
    Watchout.MyEntity;

    z.lazy(() => z.tuple([Watchout.MyEntity])).parse([...arguments]);
    return client.request("Watchout", "nested", arguments);
  }
}
export namespace Pixera {
  export const MyEntity2 = z.lazy(() => z.object({ value: Rs.Usize }));
  export interface MyEntity2 {
    value: Rs.Usize;
  }

  // @ts-ignore
  export function x(client: Rs.Client): Store<String> {
    z.lazy(() => z.tuple([])).parse([...arguments]);
    return client.subscribe("Pixera", "x", arguments);
  }
}
