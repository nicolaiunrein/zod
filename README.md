# Remotely
**simple and safe typescript RPC for rust**

## Overview
Remotely generates all the code neccessary to call rust functions from typescript.
It also generates bindings using the `zod` typescript library.

## TODO
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
- [ ] implement missing tags on newtype enums containing objects
- [ ] RPC macros



## Points to consider?
- flattening a `std::collections::HashMap` onto a struct. This works in serde but not in zod because we represent the `HashMap` as a `z.map([..])` which represents a `Map` in ts/js.
