import { z } from "zod";

export namespace Rs {
    export type String = string;

    export const String = z.string();

    export type Tuple3<T1, T2, T3> = [T1, T2, T3];

    export const Tuple3 = (T1: z.ZodTypeAny, T2: z.ZodTypeAny, T3: z.ZodTypeAny) => z.tuple([T1, T2, T3]);

    export type U16 = number;

    export const U16 = z.number().finite().int().nonnegative().lte(65535);

    export type U8 = number;

    export const U8 = z.number().finite().int().nonnegative().lte(255);

    export type Unit = null;

    export const Unit = z.null();



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

// (
//     {
//         Export {
//             docs: None,
//             path: Path {
//                 ns: "Chat",
//                 name: "MyNewtype",
//                 generic: None,
//             },
//             schema: Object(
//                 ObjectSchema {
//                     fields: [
//                         NamedField {
//                             name: "t1",
//                             value: Generic {
//                                 name: "T1",
//                             },
//                             optional: false,
//                         },
//                         NamedField {
//                             name: "t2",
//                             value: Generic {
//                                 name: "T2",
//                             },
//                             optional: false,
//                         },
//                         NamedField {
//                             name: "list",
//                             value: Resolved {
//                                 path: Path {
//                                     ns: "Rs",
//                                     name: "Tuple3",
//                                     generic: None,
//                                 },
//                                 args: [
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "String",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "U16",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "U8",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                 ],
//                             },
//                             optional: false,
//                         },
//                     ],
//                     generics: [
//                         "T1",
//                         "T2",
//                     ],
//                     extends: [],
//                 },
//             ),
//         },
//         Export {
//             docs: None,
//             path: Path {
//                 ns: "Chat",
//                 name: "MyNewtype",
//                 generic: None,
//             },
//             schema: Object(
//                 ObjectSchema {
//                     fields: [
//                         NamedField {
//                             name: "t1",
//                             value: Generic {
//                                 name: "T1",
//                             },
//                             optional: false,
//                         },
//                         NamedField {
//                             name: "t2",
//                             value: Generic {
//                                 name: "T2",
//                             },
//                             optional: false,
//                         },
//                         NamedField {
//                             name: "list",
//                             value: Resolved {
//                                 path: Path {
//                                     ns: "Rs",
//                                     name: "Tuple3",
//                                     generic: None,
//                                 },
//                                 args: [
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "U8",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "String",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "U8",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                 ],
//                             },
//                             optional: false,
//                         },
//                     ],
//                     generics: [
//                         "T1",
//                         "T2",
//                     ],
//                     extends: [],
//                 },
//             ),
//         },
//         Export {
//             docs: None,
//             path: Path {
//                 ns: "Chat",
//                 name: "MyNewtype",
//                 generic: None,
//             },
//             schema: Object(
//                 ObjectSchema {
//                     fields: [
//                         NamedField {
//                             name: "t1",
//                             value: Generic {
//                                 name: "T1",
//                             },
//                             optional: false,
//                         },
//                         NamedField {
//                             name: "t2",
//                             value: Generic {
//                                 name: "T2",
//                             },
//                             optional: false,
//                         },
//                         NamedField {
//                             name: "list",
//                             value: Resolved {
//                                 path: Path {
//                                     ns: "Rs",
//                                     name: "Tuple3",
//                                     generic: None,
//                                 },
//                                 args: [
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "U16",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "String",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "U8",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                 ],
//                             },
//                             optional: false,
//                         },
//                     ],
//                     generics: [
//                         "T1",
//                         "T2",
//                     ],
//                     extends: [],
//                 },
//             ),
//         },
//         Export {
//             docs: None,
//             path: Path {
//                 ns: "Chat",
//                 name: "MyNewtype",
//                 generic: None,
//             },
//             schema: Object(
//                 ObjectSchema {
//                     fields: [
//                         NamedField {
//                             name: "t1",
//                             value: Generic {
//                                 name: "T1",
//                             },
//                             optional: false,
//                         },
//                         NamedField {
//                             name: "t2",
//                             value: Generic {
//                                 name: "T2",
//                             },
//                             optional: false,
//                         },
//                         NamedField {
//                             name: "list",
//                             value: Resolved {
//                                 path: Path {
//                                     ns: "Rs",
//                                     name: "Tuple3",
//                                     generic: None,
//                                 },
//                                 args: [
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "String",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "U8",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "U8",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                 ],
//                             },
//                             optional: false,
//                         },
//                     ],
//                     generics: [
//                         "T1",
//                         "T2",
//                     ],
//                     extends: [],
//                 },
//             ),
//         },
//         Export {
//             docs: None,
//             path: Path {
//                 ns: "Chat",
//                 name: "MyNewtype2",
//                 generic: None,
//             },
//             schema: Object(
//                 ObjectSchema {
//                     fields: [
//                         NamedField {
//                             name: "inner",
//                             value: Resolved {
//                                 path: Path {
//                                     ns: "Chat",
//                                     name: "MyNewtype",
//                                     generic: None,
//                                 },
//                                 args: [
//                                     Resolved {
//                                         path: Path {
//                                             ns: "",
//                                             name: "",
//                                             generic: Some(
//                                                 0,
//                                             ),
//                                         },
//                                         args: [],
//                                     },
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "String",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                 ],
//                             },
//                             optional: false,
//                         },
//                         NamedField {
//                             name: "flipped",
//                             value: Resolved {
//                                 path: Path {
//                                     ns: "Chat",
//                                     name: "MyNewtype",
//                                     generic: None,
//                                 },
//                                 args: [
//                                     Resolved {
//                                         path: Path {
//                                             ns: "Rs",
//                                             name: "String",
//                                             generic: None,
//                                         },
//                                         args: [],
//                                     },
//                                     Resolved {
//                                         path: Path {
//                                             ns: "",
//                                             name: "",
//                                             generic: Some(
//                                                 0,
//                                             ),
//                                         },
//                                         args: [],
//                                     },
//                                 ],
//                             },
//                             optional: false,
//                         },
//                     ],
//                     generics: [
//                         "T",
//                     ],
//                     extends: [],
//                 },
//             ),
//         },
//     },
//     {
//         RpcRequest {
//             path: Path {
//                 ns: "Chat",
//                 name: "debug",
//                 generic: None,
//             },
//             args: [
//                 NamedField {
//                     name: "t",
//                     value: Resolved {
//                         path: Path {
//                             ns: "Chat",
//                             name: "MyNewtype2",
//                             generic: None,
//                         },
//                         args: [
//                             Resolved {
//                                 path: Path {
//                                     ns: "Rs",
//                                     name: "U8",
//                                     generic: None,
//                                 },
//                                 args: [],
//                             },
//                         ],
//                     },
//                     optional: false,
//                 },
//                 NamedField {
//                     name: "t2",
//                     value: Resolved {
//                         path: Path {
//                             ns: "Chat",
//                             name: "MyNewtype2",
//                             generic: None,
//                         },
//                         args: [
//                             Resolved {
//                                 path: Path {
//                                     ns: "Rs",
//                                     name: "U16",
//                                     generic: None,
//                                 },
//                                 args: [],
//                             },
//                         ],
//                     },
//                     optional: false,
//                 },
//             ],
//             output: Resolved {
//                 path: Path {
//                     ns: "Rs",
//                     name: "Unit",
//                     generic: None,
//                 },
//                 args: [],
//             },
//             kind: Method,
//         },
//     },
// )
export namespace Chat {
    export interface MyNewtype<T1, T2> { t1: T1, t2: T2, list: Rs.Tuple3<Rs.String, Rs.U16, Rs.U8> }

    export const MyNewtype = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.object({ t1: T1, t2: T2, list: Rs.Tuple3(Rs.String, Rs.U16, Rs.U8) });

    export interface MyNewtype<T1, T2> { t1: T1, t2: T2, list: Rs.Tuple3<Rs.U8, Rs.String, Rs.U8> }

    export const MyNewtype = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.object({ t1: T1, t2: T2, list: Rs.Tuple3(Rs.U8, Rs.String, Rs.U8) });

    export interface MyNewtype<T1, T2> { t1: T1, t2: T2, list: Rs.Tuple3<Rs.U16, Rs.String, Rs.U8> }

    export const MyNewtype = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.object({ t1: T1, t2: T2, list: Rs.Tuple3(Rs.U16, Rs.String, Rs.U8) });

    export interface MyNewtype<T1, T2> { t1: T1, t2: T2, list: Rs.Tuple3<Rs.String, Rs.U8, Rs.U8> }

    export const MyNewtype = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.object({ t1: T1, t2: T2, list: Rs.Tuple3(Rs.String, Rs.U8, Rs.U8) });

    export interface MyNewtype2<T> { inner: Chat.MyNewtype<T, Rs.String>, flipped: Chat.MyNewtype<Rs.String, T> }

    export const MyNewtype2 = (T: z.ZodTypeAny) => z.object({ inner: Chat.MyNewtype(T, Rs.String), flipped: Chat.MyNewtype(Rs.String, T) });

    export function init(client: Rs.Client) {
        const __zod_private_client_instance = client;

        return {// @ts-ignore
            async debug(t: Chat.MyNewtype2<Rs.U8>, t2: Chat.MyNewtype2<Rs.U16>): Promise<Rs.Unit> {
                return Rs.Unit.parse(await __zod_private_client_instance.call("Chat", "debug", z.lazy(() => z.tuple([Chat.MyNewtype2(Rs.U8), Chat.MyNewtype2(Rs.U16)])).parse([t, t2])));
            },

        }
    }
}


