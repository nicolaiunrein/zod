# zod

**simply derive your `typescript` / `Rust` interop!**

[![ci status](https://github.com/nicolaiunrein/zod/workflows/CI/badge.svg)](https://github.com/nicolaiunrein/zod/workflows/CI)
[![Unsafe Rust forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg?style=flat-square&logo=rust)](https://github.com/rust-secure-code/safety-dance/)
[![Crates.io version](https://img.shields.io/crates/v/zod.svg?style=flat-square)](https://crates.io/crates/zod)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/zod)
[![downloads](https://img.shields.io/crates/d/zod.svg?style=flat-square)](https://crates.io/crates/zod)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&logo=pr)](https://github.com/nicolaiunrein/zod/compare)

### Overview
This crate generates [zod](https://github.com/colinhacks/zod) bindings for your `rust` types and
optionally generates an sdk fully automatically.

*please see [docs](https://docs.rs/zod) for more details*

### Example
```rust
#[derive(Namespace)]
struct Ns;

#[derive(Serialize, Deserialize, Zod)]
struct MyStruct {
    port: u16,
    data: MyData
}

#[derive(Serialize, Deserialize, Zod)]
enum MyData {
    Hello(String),
    World(Vec<usize>)
}

```
Deriving Zod implements the [ZodType](https://docs.rs/zod-core/ZodType) trait for you exposing a couple of methods to the
typescript/schema representation of your rust types.

Calling `MyStruct::schema()` will give you a string with a valid zod schema definition:
```ts
z.object({ port: Rs.U16, data: Ns.MyData })
```

Respectively `MyData::schema()` will give you:

```ts
z.discriminatedUnion([
   z.object({ Hello: Rs.String }),
   z.object({ World: z.array(Rs.Usize) })
])
```

There is also the `type_def` method which will give you a typescript type definition:
```ts
{ port: Rs.U16, data: Ns.MyData }
```
and

```ts
{ Hello: Rs.String } | { World: [Rs.Usize] }
```

### TODO
- [x] Codegen for struct style enums
- [x] implement all missing serde attrs where possible. see: [ts-rs](https://docs.rs/ts-rs/latest/ts_rs/)

   - [x] rename
   - [x] rename-all
   - [x] tag
        - [x] internally
        - [x] externally
        - [x] adjacently
        - [x] untagged
   - [x] skip
   - [x] skip_deserializing
   - [x] default
   - [x] transparent structs
   - [x] flatten

- [x] implement tuple structs as z.tuple()
- [x] Restrict non-default fields in tuple structs to only come before the first default field
- [x] create namespace macro
- [x] RPC macros
- [ ] Consider to allow the use of generics otherwise force implementors to not have generics
- [ ] RPC ui tests
- [ ] codegen options (eg. schema prefix/suffix, type prefix/suffix)
- [ ] improve diagnostics on rpc (eg. correct spans, better compile time errors)
- [ ] improve macro hygiene
    - [ ] use crate_name in zod-derive
    - [ ] const scope where possible

- [ ] add integration tests with jest
- [ ] add a mapping table to README.md
- [ ] write detailed intro
- [ ] write rust-docs
- [ ] consider making Result/Option "smart" classes with methods like `unwrap`, `map`, `is_ok`, `is_none` etc.
- [ ] make rpc a feature or consider splitting the crates entirely
- [ ] consider supporting nested namespaces

### Considerations
- flattening a `std::collections::HashMap` onto a struct. This works in serde but not in zod because we represent the `HashMap` as a `z.map([..])` which represents a `Map` in ts/js.

### Contributing
Contribution is more than welcome. This crate is extensively tested but there are a lot of edge-cases. If you find anything that is not working but should, please let meknow.

## Type Overview



[usize](usize)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative()

```
[u8](u8)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative().lte(255)

```
[u16](u16)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative().lte(65535)

```
[u32](u32)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative().lte(4294967295)

```
[u64](u64)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative().lte(18446744073709551615)

```
[u128](u128)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative().lte(340282366920938463463374607431768211455)

```
[i8](i8)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().lte(127).gte(-128)

```
[i16](i16)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().lte(32767).gte(-32768)

```
[i32](i32)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().lte(2147483647).gte(-2147483648)

```
[i64](i64)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().lte(9223372036854775807).gte(-9223372036854775808)

```
[i128](i128)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().lte(170141183460469231731687303715884105727).gte(-170141183460469231731687303715884105728)

```
[usize](usize)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative()

```
[isize](isize)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int()

```
[f32](f32)
```ts
// TS-type
number

// inlined:
number

// schema
z.number()

```
[f64](f64)
```ts
// TS-type
number

// inlined:
number

// schema
z.number()

```
[String]
```ts
// TS-type
string

// inlined:
string

// schema
z.string()

```
[&str](&str)
```ts
// TS-type
string

// inlined:
string

// schema
z.string()

```
[bool](bool)
```ts
// TS-type
boolean

// inlined:
boolean

// schema
z.bool()

```
[char](char)
```ts
// TS-type
string

// inlined:
string

// schema
z.string().length(1)

```
[()](https://doc.rust-lang.org/std/primitive.unit.html)
```ts
// TS-type
null

// inlined:
null

// schema
z.null()

```
[Box&lt;T&gt;](Box)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Arc&lt;T&gt;](std::sync::Arc)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Rc&lt;T&gt;](std::rc::Rc)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Cow&lt;'static, T&gt;](std::borrow::Cow)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Cell&lt;T&gt;](std::cell::Cell)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[RefCell&lt;T&gt;](std::cell::RefCell)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Mutex&lt;T&gt;](std::sync::Mutex)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Weak&lt;T&gt;](std::sync::Weak)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[PhantomData&lt;T&gt;](std::marker::PhantomData)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[HashSet&lt;T&gt;](std::collections::HashSet)
```ts
// TS-type
Set&lt;T&gt;

// inlined:
Set&lt;T&gt;

// schema
z.set(T)

```
[HashMap&lt;T1, U2&gt;](std::collections::HashMap)
```ts
// TS-type
Map&lt;T, U&gt;

// inlined:
Map&lt;T, U&gt;

// schema
z.map(T, U)

```
[BTreeSet&lt;T&gt;](std::collections::BTreeSet)
```ts
// TS-type
Set&lt;T&gt;

// inlined:
Set&lt;T&gt;

// schema
z.set(T)

```
[BTreeMap&lt;T1, U2&gt;](std::collections::BTreeMap)
```ts
// TS-type
Map&lt;T, U&gt;

// inlined:
Map&lt;T, U&gt;

// schema
z.map(T, U)

```
[Vec&lt;T&gt;](Vec)
```ts
// TS-type
Array&lt;T&gt;

// inlined:
Array&lt;T&gt;

// schema
z.array(T)

```
[Option&lt;T&gt;](Option)
```ts
// TS-type
(T &#124; undefined)

// inlined:
(T &#124; undefined)

// schema
T.optional()

```
[Result&lt;T, E&gt;](Result)
```ts
// TS-type
{ Ok: T } &#124; { Err: U }

// inlined:
{ Ok: T } &#124; { Err: U }

// schema
z.union([z.object({ Ok: T }), z.object({ Err: U })])

```

## Type Overview



[usize](usize)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative()

```
[u8](u8)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative().lte(255)

```
[u16](u16)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative().lte(65535)

```
[u32](u32)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative().lte(4294967295)

```
[u64](u64)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative().lte(18446744073709551615)

```
[u128](u128)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative().lte(340282366920938463463374607431768211455)

```
[i8](i8)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().lte(127).gte(-128)

```
[i16](i16)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().lte(32767).gte(-32768)

```
[i32](i32)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().lte(2147483647).gte(-2147483648)

```
[i64](i64)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().lte(9223372036854775807).gte(-9223372036854775808)

```
[i128](i128)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().lte(170141183460469231731687303715884105727).gte(-170141183460469231731687303715884105728)

```
[usize](usize)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int().nonnegative()

```
[isize](isize)
```ts
// TS-type
number

// inlined:
number

// schema
z.number().finite().int()

```
[f32](f32)
```ts
// TS-type
number

// inlined:
number

// schema
z.number()

```
[f64](f64)
```ts
// TS-type
number

// inlined:
number

// schema
z.number()

```
[String]
```ts
// TS-type
string

// inlined:
string

// schema
z.string()

```
[&str](&str)
```ts
// TS-type
string

// inlined:
string

// schema
z.string()

```
[bool](bool)
```ts
// TS-type
boolean

// inlined:
boolean

// schema
z.bool()

```
[char](char)
```ts
// TS-type
string

// inlined:
string

// schema
z.string().length(1)

```
[()](https://doc.rust-lang.org/std/primitive.unit.html)
```ts
// TS-type
null

// inlined:
null

// schema
z.null()

```
[Box&lt;T&gt;](Box)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Arc&lt;T&gt;](std::sync::Arc)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Rc&lt;T&gt;](std::rc::Rc)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Cow&lt;'static, T&gt;](std::borrow::Cow)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Cell&lt;T&gt;](std::cell::Cell)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[RefCell&lt;T&gt;](std::cell::RefCell)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Mutex&lt;T&gt;](std::sync::Mutex)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[Weak&lt;T&gt;](std::sync::Weak)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[PhantomData&lt;T&gt;](std::marker::PhantomData)
```ts
// TS-type
T

// inlined:
T

// schema
T

```
[HashSet&lt;T&gt;](std::collections::HashSet)
```ts
// TS-type
Set&lt;T&gt;

// inlined:
Set&lt;T&gt;

// schema
z.set(T)

```
[HashMap&lt;T1, U2&gt;](std::collections::HashMap)
```ts
// TS-type
Map&lt;T, U&gt;

// inlined:
Map&lt;T, U&gt;

// schema
z.map(T, U)

```
[BTreeSet&lt;T&gt;](std::collections::BTreeSet)
```ts
// TS-type
Set&lt;T&gt;

// inlined:
Set&lt;T&gt;

// schema
z.set(T)

```
[BTreeMap&lt;T1, U2&gt;](std::collections::BTreeMap)
```ts
// TS-type
Map&lt;T, U&gt;

// inlined:
Map&lt;T, U&gt;

// schema
z.map(T, U)

```
[Vec&lt;T&gt;](Vec)
```ts
// TS-type
Array&lt;T&gt;

// inlined:
Array&lt;T&gt;

// schema
z.array(T)

```
[Option&lt;T&gt;](Option)
```ts
// TS-type
(T &#124; undefined)

// inlined:
(T &#124; undefined)

// schema
T.optional()

```
[Result&lt;T, E&gt;](Result)
```ts
// TS-type
{ Ok: T } &#124; { Err: U }

// inlined:
{ Ok: T } &#124; { Err: U }

// schema
z.union([z.object({ Ok: T }), z.object({ Err: U })])

```

