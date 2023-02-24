# Remotely
**simple and safe typescript RPC for rust**

## Overview
Remotely generates all the code neccessary to call rust functions from typescript.
It also generates bindings using the `zod` typescript library.

## TODO
- [x] Codegen for struct style enums
- [ ] RPC macros
- [ ] implement serde on newtype enums containing objects
- [ ] implement all missing serde attrs where possible. see: [ts-rs](https://docs.rs/ts-rs/latest/ts_rs/)

   - [ ] rename
   - [ ] rename-all
   - [ ] tag
        - [x] internally
        - [x] externally
        - [ ] adjacently
        - [ ] untagged
   - [ ] content
   - [ ] untagged
   - [ ] skip
   - [ ] skip_serializing
   - [ ] skip_deserializing
   - [ ] skip_serializing_if = "Option::is_none"
   - [ ] flatten
   - [ ] default
