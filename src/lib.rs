//! **Derive your `typescript` / `Rust` interop**

//! [![CI](https://github.com/nicolaiunrein/zod/actions/workflows/ci.yml/badge.svg)](https://github.com/nicolaiunrein/zod/actions/workflows/ci.yml)
//! [![Unsafe Rust forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg?style=flat-square&logo=rust)](https://github.com/rust-secure-code/safety-dance/)
//! [![Crates.io version](https://img.shields.io/crates/v/zod.svg?style=flat-square)](https://crates.io/crates/zod)
//! [![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/zod)
//! [![downloads](https://img.shields.io/crates/d/zod.svg?style=flat-square)](https://crates.io/crates/zod)
//! [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&logo=pr)](https://github.com/nicolaiunrein/zod/compare)
//!
//! This crate provides integrations with the [zod](https://github.com/colinhacks/zod) typescript library.
//!
//! **_NOTE:_**  This crate is not ready for production yet!
//!
//! ## Overview
//! This crate generates [zod](https://github.com/colinhacks/zod) bindings for your `rust` types and
//! optionally generates an sdk fully automatically.
//!
//! ## Goals:
//! ### Developer Experience:
//! Generate a fully featured typescript SDK by annotating your rust methods with some macros.
//!
//! ### Framework agnostic:
//! We provide adapters for axum servers and native websockets on the client.
//! It allows custom client/server implementations, making it compatible with all JavaScript frameworks and runtimes.
//!
//! ## TODO
//! - [x] Codegen for struct style enums
//! - [x] implement all missing serde attrs where possible. see: [ts-rs](https://docs.rs/ts-rs/latest/ts_rs/)
//!
//!    - [x] rename
//!    - [x] rename-all
//!    - [x] tag
//!         - [x] internally
//!         - [x] externally
//!         - [x] adjacently
//!         - [x] untagged
//!    - [x] skip
//!    - [x] skip_deserializing
//!    - [x] default
//!    - [x] transparent structs
//!    - [x] flatten
//!
//! - [x] implement tuple structs as z.tuple()
//! - [x] Restrict non-default fields in tuple structs to only come before the first default field
//! - [x] create namespace macro
//! - [x] RPC macros
//! - [x] codegen options (eg. schema prefix/suffix, type prefix/suffix)
//! - [x] add a mapping table to README.md
//! - [x] write detailed intro
//! - [x] write rust-docs
//! - [x] Consider to allow the use of generics otherwise force implementors to not have generics
//! - [x] RPC ui tests
//! - [ ] improve diagnostics on rpc (eg. correct spans, better compile time errors)
//! - [x] improve macro hygiene
//!     - [x] use crate_name in zod-derive
//!     - [x] const scope where possible
//!
//! - [ ] add integration tests with jest
//! - [ ] consider making Result/Option "smart" classes with methods like `unwrap`, `map`, `is_ok`, `is_none` etc.
//! - [ ] add camelCasing for method names
//!
//! ## Consideration?
//! - flattening a `std::collections::HashMap` onto a struct. This works in serde but not in zod because we represent the `HashMap` as a `z.map([..])` which represents a `Map` in ts/js.
//!
//! ## Contributing
//! Contribution is more than welcome. This crate is extensively tested but there are a lot of edge-cases. If you find anything that is not working but should, please let meknow.
//!
//!
//!
#![deny(unsafe_code)]

pub use zod_core::types;
pub use zod_core::RequestType;
pub use zod_core::ResponseType;

pub use zod_derive::*;

pub mod server;

pub use zod_core as core;

#[doc(hidden)]
pub mod __private {
    pub use async_trait;
    pub use futures;
    pub use serde;
    pub use serde_json;
    pub use tokio;
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/zod/fail/*.rs");
    t.compile_fail("tests/ui/rpc/fail/*.rs");
}
