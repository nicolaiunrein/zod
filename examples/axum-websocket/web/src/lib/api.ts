import { z } from "zod";

export namespace Rs {
  export type String = string;

  export const String = z.string();

  export type Unit = null;

  export const Unit = z.null();

  export type Usize = BigInt;

  export const Usize = z.coerce
    .bigint()
    .nonnegative()
    .lt(2n ** 64n);

  export type VecDeque<T> = T[];

  export const VecDeque = (T: z.ZodTypeAny) => z.array(T);

  export interface Client {
    get_stream(ns: string, method: string, args: unknown[]): Stream<unknown>;
    call(ns: string, method: string, args: unknown[]): Promise<unknown>;
  }
  export interface Stream<T> {
    subscribe(
      next: (value: { data: T } | { err: any } | { loading: true }) => void
    ): () => void;
  }
}

export namespace Chat {
  export interface Message {
    user: Chat.User;
    color: Rs.String;
    content: Rs.String;
  }

  export const Message = z.lazy(() =>
    z.object({ user: Chat.User, color: Rs.String, content: Rs.String })
  );

  export interface User {
    name: Rs.String;
  }

  export const User = z.lazy(() => z.object({ name: Rs.String }));

  export function init(client: Rs.Client) {
    return {
      // @ts-ignore
      count_to(n: Rs.Usize): Rs.Stream<Rs.Usize> {
        z.lazy(() => z.tuple([Rs.Usize])).parse([n]);
        return {
          subscribe(cb) {
            return client
              .get_stream("Chat", "count_to", [n])
              .subscribe((val) => {
                if ("data" in val) {
                  cb({ data: Rs.Usize.parse(val.data) });
                } else if ("err" in val) {
                  cb({ err: val.err });
                }
              });
          },
        };
      },

      // @ts-ignore
      messages(len: Rs.Usize): Rs.Stream<Rs.VecDeque<Chat.Message>> {
        z.lazy(() => z.tuple([Rs.Usize])).parse([len]);
        return {
          subscribe(cb) {
            return client
              .get_stream("Chat", "messages", [len])
              .subscribe((val) => {
                if ("data" in val) {
                  cb({ data: Rs.VecDeque(Chat.Message).parse(val.data) });
                } else if ("err" in val) {
                  cb({ err: val.err });
                }
              });
          },
        };
      },

      // @ts-ignore
      async send(msg: Chat.Message): Promise<Rs.Unit> {
        z.lazy(() => z.tuple([Chat.Message])).parse([msg]);
        return Rs.Unit.parse(await client.call("Chat", "send", [msg]));
      },
    };
  }
}
