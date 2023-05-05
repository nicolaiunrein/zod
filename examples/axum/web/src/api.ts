import { z } from "zod";

export namespace Rs {
    export type String = string;
    export const String = z.string();
    export type Usize = BigInt;
    export const Usize = z.bigint().nonnegative().lt(2n ** 64n);
export interface Client {
    get_stream<T>(
      ns: string,
      method: string,
      args: IArguments
    ): {
      subscribe(next: (value: T) => void): () => void;
      close(): void;
    };
    call<T>(ns: string, method: string, args: IArguments): Promise<T>;
  }
}
export namespace Pixera {
export function init(client: Rs.Client){
type Store<T> = {
    subscribe(next: (value: T) => void): () => void;
    close(): void;
};
return {// @ts-ignore
 debug_stream(): Store<Rs.String> {
    // phantom usage
    z.lazy(() => z.tuple([])).parse([...arguments]);
    return client.get_stream("Pixera", "debug_stream", arguments);
},
// @ts-ignore
 hello_stream(num: Rs.Usize): Store<Rs.Usize> {
    // phantom usage
    num;
    z.lazy(() => z.tuple([Rs.Usize])).parse([...arguments]);
    return client.get_stream("Pixera", "hello_stream", arguments);
},
// @ts-ignore
 y(): Store<Rs.String> {
    // phantom usage
    z.lazy(() => z.tuple([])).parse([...arguments]);
    return client.get_stream("Pixera", "y", arguments);
},
}}
}
export namespace Watchout {
    export interface Generic<T, V> { value: Rs.String, t: T, v: V }
    export const Generic = (T: z.ZodTypeAny, V: z.ZodTypeAny) => z.object({ value: Rs.String, t: T, v: V });
    export interface MyEntity { value: Rs.String }
    export const MyEntity = z.lazy(() => z.object({ value: Rs.String }));
    export interface User { value: Watchout.Generic<Rs.Usize, Rs.Usize> }
    export const User = z.lazy(() => z.object({ value: Watchout.Generic(Rs.Usize, Rs.Usize) }));
export function init(client: Rs.Client){
type Store<T> = {
    subscribe(next: (value: T) => void): () => void;
    close(): void;
};
return {// @ts-ignore
async  hello(_s: Rs.String, _n: Rs.Usize): Promise<Rs.Usize> {
    // phantom usage
    _s;
    _n;
    z.lazy(() => z.tuple([Rs.String, Rs.Usize])).parse([...arguments]);
    return client.call("Watchout", "hello", arguments);
},
// @ts-ignore
async  hello1(_s: Rs.String): Promise<Rs.Usize> {
    // phantom usage
    _s;
    z.lazy(() => z.tuple([Rs.String])).parse([...arguments]);
    return client.call("Watchout", "hello1", arguments);
},
// @ts-ignore
 hello_stream(num: Rs.Usize): Store<Rs.Usize> {
    // phantom usage
    num;
    z.lazy(() => z.tuple([Rs.Usize])).parse([...arguments]);
    return client.get_stream("Watchout", "hello_stream", arguments);
},
// @ts-ignore
async  hello_user(_user: Watchout.User, _n: Rs.Usize): Promise<Rs.Usize> {
    // phantom usage
    _user;
    _n;
    z.lazy(() => z.tuple([Watchout.User, Rs.Usize])).parse([...arguments]);
    return client.call("Watchout", "hello_user", arguments);
},
// @ts-ignore
async  nested(_value: Watchout.MyEntity): Promise<Rs.Usize> {
    // phantom usage
    _value;
    z.lazy(() => z.tuple([Watchout.MyEntity])).parse([...arguments]);
    return client.call("Watchout", "nested", arguments);
},
// @ts-ignore
async  test(_user: Watchout.User, _n: Rs.Usize): Promise<Rs.Usize> {
    // phantom usage
    _user;
    _n;
    z.lazy(() => z.tuple([Watchout.User, Rs.Usize])).parse([...arguments]);
    return client.call("Watchout", "test", arguments);
},
}}
}

