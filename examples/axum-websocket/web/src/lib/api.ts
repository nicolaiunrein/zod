import { z } from "zod";

export namespace Rs {
    export type F64 = number;

    export const F64 = z.number();

    export type HashMap<K, V> = Map<K, V>;

    export const HashMap = (K: z.ZodTypeAny, V: z.ZodTypeAny) => z.map(K, V);

    export type String = string;

    export const String = z.string();

    export type U16 = number;

    export const U16 = z.number().finite().int().nonnegative().lte(65535);

    export type U8 = number;

    export const U8 = z.number().finite().int().nonnegative().lte(255);

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

    export type StreamEvent<T> = { data: T } | { error: Error } | { loading: true };


}

export namespace Chat {
    export interface Message { user: Chat.User, color: Rs.String, content: Rs.String }

    export const Message = z.lazy(() => z.object({ user: Chat.User, color: Rs.String, content: Rs.String }));

    export type MyNewtype<T> = Rs.HashMap<Rs.String, T>;

    export const MyNewtype = (T: z.ZodTypeAny) => z.lazy(() => Rs.HashMap(Rs.String, T));


    export interface User { name: Rs.String }

    export const User = z.lazy(() => z.object({ name: Rs.String }));

    export function init(client: Rs.Client) {
        const __zod_private_client_instance = client;

        return {// @ts-ignore
            count_to(n: Rs.Usize): Rs.Stream<Rs.Usize> {
                return {
                    subscribe(next) {
                        return __zod_private_client_instance
                            .get_stream("Chat", "count_to", [n])
                            .subscribe((evt) => {
                                if ("data" in evt) {
                                    let result = Rs.Usize.safeParse(evt.data);
                                    if (result.success) {
                                        next({ data: result.data });
                                    } else {
                                        next({ error: result.error })
                                    }
                                } else {
                                    next(evt);
                                }
                            });
                    }
                }
            },

            // @ts-ignore
            async debug(t: Chat.MyNewtype<Rs.U8>, t2: Chat.MyNewtype<Rs.U16>): Promise<Rs.Unit> {
                return Rs.Unit.parse(await __zod_private_client_instance.call("Chat", "debug", z.lazy(() => z.tuple([Chat.MyNewtype(Rs.U8), Chat.MyNewtype(Rs.U16)])).parse([t, t2])));
            },

            // @ts-ignore
            async get_lightness(color: Rs.String): Promise<Rs.F64> {
                return Rs.F64.parse(await __zod_private_client_instance.call("Chat", "get_lightness", z.lazy(() => z.tuple([Rs.String])).parse([color])));
            },

            // @ts-ignore
            async get_random_color(): Promise<Rs.String> {
                return Rs.String.parse(await __zod_private_client_instance.call("Chat", "get_random_color", z.lazy(() => z.tuple([])).parse([])));
            },

            // @ts-ignore
            messages(len: Rs.Usize): Rs.Stream<Rs.VecDeque<Chat.Message>> {
                return {
                    subscribe(next) {
                        return __zod_private_client_instance
                            .get_stream("Chat", "messages", [len])
                            .subscribe((evt) => {
                                if ("data" in evt) {
                                    let result = Rs.VecDeque(Chat.Message).safeParse(evt.data);
                                    if (result.success) {
                                        next({ data: result.data });
                                    } else {
                                        next({ error: result.error })
                                    }
                                } else {
                                    next(evt);
                                }
                            });
                    }
                }
            },

            // @ts-ignore
            async pending(): Promise<Rs.Unit> {
                return Rs.Unit.parse(await __zod_private_client_instance.call("Chat", "pending", z.lazy(() => z.tuple([])).parse([])));
            },

            // @ts-ignore
            async send(msg: Chat.Message): Promise<Rs.Unit> {
                return Rs.Unit.parse(await __zod_private_client_instance.call("Chat", "send", z.lazy(() => z.tuple([Chat.Message])).parse([msg])));
            },

        }
    }
}


