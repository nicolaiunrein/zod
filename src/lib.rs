//! **Derive your `typescript` / `Rust` interop**

//! [![ci status](https://github.com/nicolaiunrein/zod/workflows/CI/badge.svg)](https://github.com/nicolaiunrein/zod/workflows/CI)
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
//! ## Example
//! ```rust
//! # use zod::Namespace;
//! # use serde::{Serialize, Deserialize};
//! # use zod::Zod;
//! #[derive(Namespace)]
//! struct Ns;
//!
//! #[derive(Serialize, Deserialize, Zod)]
//! #[zod(namespace = "Ns")]
//! struct MyStruct {
//!     port: u16,
//!     data: MyData
//! }
//!
//! #[derive(Serialize, Deserialize, Zod)]
//! #[zod(namespace = "Ns")]
//! enum MyData {
//!     Hello(String),
//!     World(Vec<usize>)
//! }
//!
//! ```
//! Deriving Zod implements the [ZodType](https://docs.rs/zod-core/ZodType) trait for you exposing a couple of methods to the
//! typescript/schema representation of your rust types.
//!
//! Calling `MyStruct::schema()` will give you a string with a valid zod schema definition:
//! ```ts
//! z.object({ port: Rs.U16, data: Ns.MyData })
//! ```
//!
//! Respectively `MyData::schema()` will give you:
//!
//! ```ts
//! z.discriminatedUnion([
//!    z.object({ Hello: Rs.String }),
//!    z.object({ World: z.array(Rs.Usize) })
//! ])
//! ```
//!
//! There is also the `type_def` method which will give you a typescript type definition:
//! ```ts
//! { port: Rs.U16, data: Ns.MyData }
//! ```
//! and
//!
//! ```ts
//! { Hello: Rs.String } | { World: [Rs.Usize] }
//! ```
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
//!
//! - [ ] Consider to allow the use of generics otherwise force implementors to not have generics
//! - [ ] RPC ui tests
//! - [ ] improve diagnostics on rpc (eg. correct spans, better compile time errors)
//! - [ ] improve macro hygiene
//!     - [ ] use crate_name in zod-derive
//!     - [ ] const scope where possible
//!
//! - [ ] add integration tests with jest
//! - [ ] consider making Result/Option "smart" classes with methods like `unwrap`, `map`, `is_ok`, `is_none` etc.
//! - [ ] make rpc a feature or consider splitting the crates entirely
//!
//! ## Consideration?
//! - flattening a `std::collections::HashMap` onto a struct. This works in serde but not in zod because we represent the `HashMap` as a `z.map([..])` which represents a `Map` in ts/js.
//!
//! ## Contributing
//! Contribution is more than welcome. This crate is extensively tested but there are a lot of edge-cases. If you find anything that is not working but should, please let meknow.
//!
//!
//!
#![doc = include_str!(concat!(env!("OUT_DIR"), "/type-table.md"))]
#![deny(unsafe_code)]

pub use zod_core::*;
pub use zod_derive::*;

#[cfg(feature = "rpc")]
pub mod rpc;

#[doc(hidden)]
#[cfg(feature = "rpc")]
pub mod __private {
    pub use async_trait;
    pub use futures;
    pub use inventory;
    pub use serde;
    pub use serde_json;
    pub use tokio;
    pub use tracing;
    pub use zod_core::rpc::*;
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/zod/fail/*.rs");
    t.compile_fail("tests/ui/rpc/fail/*.rs");
}
