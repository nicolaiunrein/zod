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
#![doc = include_str!("../progress.md")]
//!
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

pub use zod_core::*;
pub use zod_derive::*;

pub mod prelude {
    pub use zod_core::prelude::*;
    pub use zod_derive::*;
}

// pub mod server;

pub use zod_core as core;
