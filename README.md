# Zod
**generate [zod](https://github.com/colinhacks/zod) schemas from your Rust types and call your function from typescript clients**
## Overview
Zod generates all the code neccessary to call rust functions from typescript.
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
- [x] create namespace macro
- [x] RPC macros
- [ ] Consider to allow the use of generics otherwise force implementors to not have generics
- [ ] RPC ui tests
- [ ] codegen options (eg. schema prefix/suffix, type prefix/suffix)
- [ ] improved diagnostics on rpc (eg. correct spans, better compile time errors)
- [ ] macro hygiene
    - [ ] use crate_name in zod-derive
    - [ ] const scope where possible

- [ ] add integration tests with jest
- [ ] add a mapping table to README.md
- [ ] write detailed intro
- [ ] write rust-docs
- [ ] consider making Result/Option "smart" classes with methods like `unwrap`, `map`, `is_ok`, `is_none` etc.
- [ ] make rpc a feature or consider splitting the crates entirely

## Points to consider?
- flattening a `std::collections::HashMap` onto a struct. This works in serde but not in zod because we represent the `HashMap` as a `z.map([..])` which represents a `Map` in ts/js.

## Contributing
Contribution is more than welcome. This crate is extensively tested but there are a lot of edge-cases. If you find anything that is not working but should, please let meknow.
