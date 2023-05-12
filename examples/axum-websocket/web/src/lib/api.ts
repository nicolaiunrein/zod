import { z } from "zod";

export namespace Rs {
    export type F64 = number;

    export const F64 = z.number();

    export type String = string;

    export const String = z.string();

    export type Unit = null;

    export const Unit = z.null();

    export type Usize = BigInt;

    export const Usize = z.coerce.bigint().nonnegative().lt(2n ** 64n);

    export type VecDeque<T> = T[];

    export const VecDeque = (T: z.ZodTypeAny) => z.array(T);



    export const StreamResponse = z.object({
        stream: z.object({
            id: z.coerce.bigint().nonnegative().lt(2n ** 64n),
            data: z.unknown()
        }),
    });
    export type StreamResponse = z.infer<typeof StreamResponse>;

    export const MethodResponse = z.object({
        method: z.object({
            id: z.coerce.bigint().nonnegative().lt(2n ** 64n),
            data: z.unknown()
        })
    });
    export type MethodResponse = z.infer<typeof MethodResponse>

    export const ErrorResponse =
        z.object({
            error: z.object({
                id: z.coerce.bigint().nonnegative().lt(2n ** 64n).optional(),
                data: z.unknown()
            })
        });

    export type ErrorResponse = z.infer<typeof ErrorResponse>

    export const Response = z.union([
        StreamResponse,
        MethodResponse,
        ErrorResponse
    ])

    export type Response = z.infer<typeof Response>;

    export interface Client {
        get_stream(ns: string, method: string, args: unknown[]): Stream<unknown>;
        call(ns: string, method: string, args: unknown[]): Promise<unknown>;
    }
    export interface Stream<T> {
        subscribe(
            next: (value: StreamEvent<T>) => void
        ): () => void;
    }

    export type StreamEvent<T> = { data: T } | { error: ZodError } | { loading: true };

    export interface ZodError {
        name: "JsonError" | "UnknownNamespace",
        message: string
    }

}

export namespace Chat {
    export interface Message { user: Chat.User, color: Rs.String, content: Rs.String }

    export const Message = z.lazy(() => z.object({ user: Chat.User, color: Rs.String, content: Rs.String }));

    export interface User { name: Rs.String }

    export const User = z.lazy(() => z.object({ name: Rs.String }));

    export function init(client: Rs.Client) {
        return {// @ts-ignore
            count_to(n: Rs.Usize): Rs.Stream<Rs.Usize> {
                z.lazy(() => z.tuple([Rs.Usize])).parse([n]);
                return {
                    subscribe(cb) {
                        return client
                            .get_stream("Chat", "count_to", [n])
                            .subscribe((val) => {
                                if ("data" in val) {
                                    cb({ data: Rs.Usize.parse(val.data) });
                                } else {
                                    cb(val);
                                }
                            });
                    }
                }
            },

            // @ts-ignore
            async get_lightness(color: Rs.String): Promise<Rs.F64> {
                z.lazy(() => z.tuple([Rs.String])).parse([color]);
                return Rs.F64.parse(await client.call("Chat", "get_lightness", [color]));
            },

            // @ts-ignore
            async get_random_color(): Promise<Rs.String> {
                z.lazy(() => z.tuple([])).parse([]);
                return Rs.String.parse(await client.call("Chat", "get_random_color", []));
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
                                } else {
                                    cb(val);
                                }
                            });
                    }
                }
            },

            // @ts-ignore
            async pending(): Promise<Rs.Unit> {
                z.lazy(() => z.tuple([])).parse([]);
                return Rs.Unit.parse(await client.call("Chat", "pending", []));
            },

            // @ts-ignore
            async send(msg: Chat.Message): Promise<Rs.Unit> {
                z.lazy(() => z.tuple([Chat.Message])).parse([msg]);
                return Rs.Unit.parse(await client.call("Chat", "send", [msg]));
            },

        }
    }
}


