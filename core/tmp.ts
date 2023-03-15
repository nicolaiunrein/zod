import * as z from "zod";
export namespace Rs {
  export type Unit = null;
  export const Unit = z.null();

  export type Bool = boolean;
  export const Bool = z.bool();

  export type Char = string;
  export const Char = z.string().length(1);

  export type String = string;
  export const String = z.string();

  export type U8 = number;
  export const U8 = z.number().finite().int().nonnegative().lte(255);

  export type U16 = number;
  export const U16 = z.number().finite().int().nonnegative().lte(65535);

  export type U32 = number;
  export const U32 = z.number().finite().int().nonnegative().lte(4294967295);

  export type U64 = number;
  export const U64 = z
    .number()
    .finite()
    .int()
    .nonnegative()
    .lte(18446744073709551615);

  export type U128 = number;
  export const U128 = z
    .number()
    .finite()
    .int()
    .nonnegative()
    .lte(340282366920938463463374607431768211455);

  export type Usize = number;
  export const Usize = z.number().finite().int().nonnegative();

  export type I8 = number;
  export const I8 = z.number().finite().int().lte(127).gte(-128);

  export type I16 = number;
  export const I16 = z.number().finite().int().lte(32767).gte(-32768);

  export type I32 = number;
  export const I32 = z.number().finite().int().lte(2147483647).gte(-2147483648);

  export type I64 = number;
  export const I64 = z
    .number()
    .finite()
    .int()
    .lte(9223372036854775807)
    .gte(-9223372036854775808);

  export type I128 = number;
  export const I128 = z
    .number()
    .finite()
    .int()
    .lte(170141183460469231731687303715884105727)
    .gte(-170141183460469231731687303715884105728);

  export type Isize = number;
  export const Isize = z.number().finite().int();

  export type F32 = number;
  export const F32 = z.number();

  export type F64 = number;
  export const F64 = z.number();

  export type Ipv4Addr = string;
  export const Ipv4Addr = z.string().ip({ version: "v4" });

  export type Ipv6Addr = string;
  export const Ipv6Addr = z.string().ip({ version: "v6" });

  export type IpAddr = string;
  export const IpAddr = z.string().ip();

  export type Vec<T> = T[];
  export const Vec = (T: z.ZodTypeAny) => z.array(z.lazy(() => T));

  export type Array<N extends number, T, TObj = [T, ...T[]]> = Pick<
    TObj,
    Exclude<keyof TObj, "splice" | "push" | "pop" | "shift" | "unshift">
  > & {
    readonly length: N;
    [I: number]: T;
    [Symbol.iterator]: () => IterableIterator<T>;
  };

  export const Array = (N: number, T: z.ZodTypeAny) =>
    z.array(z.lazy(() => T)).length(N);

  export type HashSet<T> = Set<T>;
  export const HashSet = (T: z.ZodTypeAny) => z.set(z.lazy(() => T));

  export type HashMap<K, V> = Map<K, V>;
  export const HashMap = (K: z.ZodTypeAny, V: z.ZodTypeAny) =>
    z.map(
      z.lazy(() => K),
      z.lazy(() => V)
    );

  export type BTreeMap<K, V> = Map<K, V>;
  export const BTreeMap = (K: z.ZodTypeAny, V: z.ZodTypeAny) =>
    z.map(
      z.lazy(() => K),
      z.lazy(() => V)
    );

  export type Option<T> = T | undefined;
  export const Option = (T: z.ZodTypeAny) => z.lazy(() => T).optional();

  export type Result<T, E> = { Ok: T } | { Err: E };
  export const Result = (T: z.ZodTypeAny, E: z.ZodTypeError) =>
    z.union([
      z.object({ Ok: z.lazy(() => T) }),
      z.object({ Err: z.lazy(() => E) }),
    ]);

  export type Tuple1<T1> = [T1];
  export const Tuple1 = (T1: z.ZodTypeAny) => z.tuple([z.lazy(() => T1)]);

  export type Tuple2<T1, T2> = [T1, T2];
  export const Tuple2 = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) =>
    z.tuple([z.lazy(() => T1), z.lazy(() => T2)]);

  export type Tuple3<T1, T2, T3> = [T1, T2, T3];
  export const Tuple3 = (
    T1: z.ZodTypeAny,
    T2: z.ZodTypeAny,
    T3: z.ZodTypeAny
  ) => z.tuple([z.lazy(() => T1), z.lazy(() => T2), z.lazy(() => T3)]);

  export type Tuple4<T1, T2, T3, T4> = [T1, T2, T3, T4];
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

  export type Tuple5<T1, T2, T3, T4, T5> = [T1, T2, T3, T4, T5];
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

  export type Tuple6<T1, T2, T3, T4, T5, T6> = [T1, T2, T3, T4, T5, T6];
  export const Tuple = (...args: z.ZodTypeAny[]) =>
    z.tuple([...args.map((T: z.ZodTypeAny) => z.lazy(() => T))]);

  let x = Tuple([z.string(), z.number()]);

  export type Tuple66<T, T1, T2, T3, T4, T5> = [
    T,
    ...Tuple5<T1, T2, T3, T4, T5>
  ];

  export type Tuple7<T1, T2, T3, T4, T5, T6, T7> = [T1, T2, T3, T4, T5, T6, T7];
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
}
