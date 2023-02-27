# Remotely
**simple and safe typescript RPC for rust**

## Overview
Remotely generates all the code neccessary to call rust functions from typescript.
It also generates bindings using the `zod` typescript library.

## TODO
- [x] Codegen for struct style enums
- [ ] RPC macros
- [ ] implement serde on newtype enums containing objects
- [ ] consider adding tuple structs as tuples
- [ ] implement all missing serde attrs where possible. see: [ts-rs](https://docs.rs/ts-rs/latest/ts_rs/)

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
   - [ ] flatten
   - [ ] transparent


