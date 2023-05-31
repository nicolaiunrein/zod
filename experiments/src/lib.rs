//! ## Problem :
//! The impl is split between Argument types and Response types.
//! The Traits are the same. Find a way to uniform them.
//!
//!
//! ## Solution:
//! The implementor should not care.
//! We require `Type<Kind::Input>` to be implemented for RPC arguments and `Type<Kind::Output>` for Rpc response
//! types. When the exports are equal they should get merged into the io namespace when generating
//! the code. For references to still be valid either walk all references in all exports and
//! update to io namespace or transform the exports to alias the export in the io namespace:
//!
//! ```ts
//! export namespace MyNs {
//!     export namespace input {
//!         export const U8 = z.number();
//!         export type U8 = number
//!     }
//!     
//!     export namespace output {
//!         export const U8 = z.number();
//!         export type U8 = number
//!     }
//! }
//! ```
//! then becomes
//! ```ts
//! export namespace MyNs {
//!     export namespace io {
//!         export const U8 = z.number();
//!         export type U8 = number
//!     }
//!     
//!     export namespace input {
//!         export const U8 = io.U8;
//!         export type U8 = io.U8;
//!     
//!     }
//!     
//!     export namespace output {
//!         export const U8 = io.U8;
//!         export type U8 = io.U8;
//!     }
//! }
//! ```
//! ## TODO
//! - [x] Implement basic codegen with generics
//! - [] Disallow trait bounds on structs and enums
//! - [] support tuple style enums with inner objects
//! - [] Implement RPC part
//! - [] implement all missing serde attrs where possible. see: [ts-rs](https://docs.rs/ts-rs/latest/ts_rs/)
//!
//!    - [] rename
//!    - [] rename-all
//!    - [x] tag
//!         - [x] internally
//!         - [x] externally
//!         - [x] adjacently
//!         - [x] untagged
//!    - [] skip
//!    - [] skip_deserializing
//!    - [] default
//!    - [] transparent structs
//!    - [] flatten
//!
//! - [] Restrict non-default fields in tuple structs to only come before the first default field
//! - [] create namespace macro
//! - [] codegen options (eg. schema prefix/suffix, type prefix/suffix)
//! - [] write detailed intro
//! - [] write rust-docs
//! - [ ] add integration tests with jest
//! - [ ] consider making Result/Option "smart" classes with methods like `unwrap`, `map`, `is_ok`, `is_none` etc.
//! - [ ] add camelCasing for method names
//!
mod build_ins;
pub mod const_str;
pub mod derive_internals;
mod export;
mod formatter;
mod utils;
pub mod z;
pub use export::*;

#[cfg(test)]
pub mod test_utils;

use std::{
    collections::{BTreeMap, HashSet},
    fmt::Display,
    marker::PhantomData,
};

use build_ins::Rs;
use formatter::{TsFormatter, ZodFormatter};
use typed_builder::TypedBuilder;
use z::{ZodType, ZodTypeInner};

pub mod prelude {
    pub use super::z;
    pub use super::DependencyVisitor;
    pub use super::Export;
    pub use super::GenericArgument;
    pub use super::Kind;
    pub use super::Namespace;
    pub use super::Type;
    pub use super::TypeExt;
    pub use crate::formatter::Formatter;
}

#[allow(non_snake_case)]
pub mod Kind {
    #[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
    pub struct Input;

    #[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
    pub struct Output;

    /// special marker for ExportMap
    #[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
    pub(crate) enum EitherIo {
        Input,
        Output,
    }

    impl From<Input> for EitherIo {
        fn from(_: Input) -> Self {
            EitherIo::Input
        }
    }
    impl From<Output> for EitherIo {
        fn from(_: Output) -> Self {
            EitherIo::Output
        }
    }
}

pub struct GenericArgument<Io> {
    name: &'static str,
    inlined: ZodType<Io>,
}

impl<Io> GenericArgument<Io>
where
    Io: Clone,
{
    pub fn new<T: Type<Io>>(name: &'static str) -> Self {
        Self {
            name,
            inlined: T::inline(),
        }
    }

    pub fn inlined(&self) -> &ZodType<Io> {
        &self.inlined
    }
    pub fn name(&self) -> &'static str {
        &self.name
    }
}

// TODO: seal this trait
pub trait IoKind {
    const NAME: &'static str;
}

impl IoKind for Kind::Input {
    const NAME: &'static str = "input";
}
impl IoKind for Kind::Output {
    const NAME: &'static str = "output";
}
impl IoKind for Kind::EitherIo {
    const NAME: &'static str = "io";
}

pub struct DependencyVisitor<Io> {
    exports: HashSet<Export<Io>>,
}

impl<Io> DependencyVisitor<Io> {
    pub fn visit<T>(&mut self)
    where
        T: Type<Io>,
        Io: std::hash::Hash + Eq,
        Io: Clone,
    {
        if let Some(export) = T::export() {
            self.exports.insert(export);
        }
    }
}

/// # Example of a manual impl
/// ```
#[doc = include_str!("../derive/tests/manual_impl.rs")]
/// ```
pub trait Type<Io>
where
    Io: Clone,
{
    type Ns: Namespace;
    const NAME: &'static str;
    const INLINE: bool;

    /// Generate the representation of this type in the context of typescript/zod.
    fn value() -> ZodType<Io>;

    /// Recursively collect the exports of nested types
    fn visit_dependencies(_visitor: &mut DependencyVisitor<Io>);

    /// Return generics of the implementing type
    fn args() -> Vec<GenericArgument<Io>>;
}

/// Trait to prevent incorret implementation of the Type trait.
pub trait TypeExt<Io>: Type<Io>
where
    Io: Clone,
{
    fn inline() -> ZodType<Io> {
        if let Some(export) = Self::export() {
            Reference {
                name: export.name,
                ns: export.ns,
                args: Self::args()
                    .iter()
                    .map(|arg| arg.inlined().clone())
                    .collect(),
                generic_replace: None,
                _phantom: Default::default(),
            }
            .into()
        } else {
            Self::value()
        }
    }

    fn export() -> Option<Export<Io>> {
        if Self::INLINE {
            None
        } else {
            Some(Export {
                name: String::from(Self::NAME),
                ns: String::from(Self::Ns::NAME),
                args: Self::args()
                    .iter()
                    .map(|arg| arg.name())
                    .collect::<Vec<_>>(),

                value: Self::value(),
            })
        }
    }
}

impl<Io, T> TypeExt<Io> for T
where
    T: Type<Io>,
    Io: Clone,
{
}

pub trait Namespace {
    const NAME: &'static str;
}

impl<const C: char, T: const_str::Chain> Type<Kind::Input> for const_str::ConstStr<C, T> {
    type Ns = Rs;
    const NAME: &'static str = "";
    const INLINE: bool = true;

    fn value() -> ZodType<Kind::Input> {
        ZodType::builder()
            .inner(ZodTypeInner::Generic(Self::value().to_string()))
            .build()
    }

    fn visit_dependencies(_visitor: &mut DependencyVisitor<Kind::Input>) {}
    fn args() -> Vec<GenericArgument<Kind::Input>> {
        Vec::new()
    }
}

impl<const C: char, T: const_str::Chain> Type<Kind::Output> for const_str::ConstStr<C, T> {
    type Ns = Rs;
    const NAME: &'static str = "";
    const INLINE: bool = true;

    fn value() -> ZodType<Kind::Output> {
        ZodType::builder()
            .inner(ZodTypeInner::Generic(Self::value().to_string()))
            .build()
    }

    fn visit_dependencies(_visitor: &mut DependencyVisitor<Kind::Output>) {}
    fn args() -> Vec<GenericArgument<Kind::Output>> {
        Vec::new()
    }
}

#[derive(TypedBuilder, Eq, Debug, Clone, Hash)]
pub struct Reference<Io> {
    #[builder(setter(into))]
    pub name: String,

    #[builder(setter(into))]
    pub ns: String,

    #[builder(default)]
    pub args: Vec<ZodType<Io>>,

    #[builder(default, setter(skip))]
    generic_replace: Option<String>,

    #[builder(default, setter(skip))]
    _phantom: PhantomData<Io>,
}

/// Type alias into the common namespace. The fields are pub(crate) to ensure this type is only
/// constructed when processing the export map
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct Alias {
    pub(crate) name: String,
    pub(crate) ns: String,
}

impl<'a> Display for ZodFormatter<'a, Alias> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}.{}.{}",
            self.ns,
            Kind::EitherIo::NAME,
            self.name
        ))
    }
}

impl<'a> Display for TsFormatter<'a, Alias> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}.{}.{}",
            self.ns,
            Kind::EitherIo::NAME,
            self.name
        ))
    }
}

impl<'a, Io> Display for TsFormatter<'a, Reference<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref generic) = self.generic_replace {
            return f.write_str(generic);
        }
        f.write_fmt(format_args!("{}.{}.{}", self.0.ns, Io::NAME, self.0.name))?;
        if !self.0.args.is_empty() {
            let args = self.0.args.iter().map(TsFormatter).collect::<Vec<_>>();

            f.write_fmt(format_args!("<{}>", utils::Separated(", ", &args)))?;
        }
        Ok(())
    }
}

impl<'a, Io> Display for ZodFormatter<'a, Reference<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref generic) = self.generic_replace {
            return f.write_str(generic);
        }

        f.write_fmt(format_args!("{}.{}.{}", self.0.ns, Io::NAME, self.0.name))?;
        if !self.0.args.is_empty() {
            let args = self.0.args.iter().map(ZodFormatter).collect::<Vec<_>>();
            f.write_fmt(format_args!("({})", utils::Separated(", ", &args)))?;
        }
        Ok(())
    }
}

impl<T> From<Reference<T>> for ZodTypeInner<T> {
    fn from(value: Reference<T>) -> Self {
        ZodTypeInner::Reference(value)
    }
}

impl From<Reference<Kind::Input>> for Reference<Kind::EitherIo> {
    fn from(other: Reference<Kind::Input>) -> Self {
        Self {
            name: other.name,
            ns: other.ns,
            args: other.args.into_iter().map(|arg| arg.into()).collect(),
            generic_replace: other.generic_replace,
            _phantom: PhantomData,
        }
    }
}

impl From<Reference<Kind::Output>> for Reference<Kind::EitherIo> {
    fn from(other: Reference<Kind::Output>) -> Self {
        Self {
            name: other.name,
            ns: other.ns,
            args: other.args.into_iter().map(|arg| arg.into()).collect(),
            generic_replace: other.generic_replace,
            _phantom: PhantomData,
        }
    }
}

impl<A, B> PartialEq<Reference<A>> for Reference<B> {
    fn eq(&self, other: &Reference<A>) -> bool {
        let Self {
            name,
            ns,
            args,
            generic_replace,
            _phantom,
        } = self;

        name == &other.name
            && ns == &other.ns
            && args == &other.args
            && generic_replace == &other.generic_replace
    }
}

macro_rules! make_eq {
    ($name: ident { $($fields: ident),* }) => {
        impl<A, B> PartialEq<$name<A>> for $name<B> {
            fn eq(&self, other: &$name<A>) -> bool {
                let Self {
                    $($fields),*
                } = self;

                $($fields == &other.$fields)&&*
            }
        }
    }
}
pub(crate) use make_eq;

struct NsMap {
    input: BTreeMap<String, Export<Kind::Input>>,
    output: BTreeMap<String, Export<Kind::Output>>,
    io: BTreeMap<String, Export<Kind::EitherIo>>,
}

impl NsMap {
    fn insert_input(&mut self, name: String, mut input: Export<Kind::Input>) {
        if let Some(output) = self.output.get_mut(&name) {
            if &mut input == output {
                let merged = Export::<Kind::EitherIo>::from(input.clone());

                let alias = Alias {
                    name: merged.name.clone(),
                    ns: merged.ns.clone(),
                };

                input.value = ZodTypeInner::Alias(alias.clone()).into();
                output.value = ZodTypeInner::Alias(alias).into();
                self.io.insert(name.clone(), merged);
            }
        }
        self.input.insert(name, input);
    }

    fn insert_output(&mut self, name: String, mut output: Export<Kind::Output>) {
        if let Some(input) = self.input.get_mut(&name) {
            if &mut output == input {
                let merged = Export::<Kind::EitherIo>::from(output.clone());

                let alias = Alias {
                    name: merged.name.clone(),
                    ns: merged.ns.clone(),
                };

                output.value = ZodTypeInner::Alias(alias.clone()).into();
                input.value = ZodTypeInner::Alias(alias).into();
                self.io.insert(name.clone(), merged);
            }
        }
        self.output.insert(name, output);
    }
}

pub struct ExportMap(BTreeMap<String, NsMap>);

impl ExportMap {
    pub fn new(
        input_exports: impl IntoIterator<Item = Export<Kind::Input>>,
        output_exports: impl IntoIterator<Item = Export<Kind::Output>>,
    ) -> Self {
        let mut out = BTreeMap::<String, NsMap>::new();

        for export in input_exports.into_iter() {
            let ns_map = out.entry(export.ns.clone()).or_insert_with(|| NsMap {
                input: Default::default(),
                output: Default::default(),
                io: Default::default(),
            });

            ns_map.insert_input(export.name.clone(), export);
        }

        for export in output_exports.into_iter() {
            let ns_map = out.entry(export.ns.clone()).or_insert_with(|| NsMap {
                input: Default::default(),
                output: Default::default(),
                io: Default::default(),
            });

            ns_map.insert_output(export.name.clone(), export);
        }

        Self(out)
    }
}

impl Display for ExportMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (ns, inner) in self.0.iter() {
            f.write_fmt(format_args!("export namespace {ns} {{\n{}}}\n", inner))?;
        }
        Ok(())
    }
}

impl Display for NsMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_part<T>(
            f: &mut std::fmt::Formatter<'_>,
            set: &BTreeMap<String, Export<T>>,
        ) -> std::fmt::Result
        where
            T: IoKind,
        {
            let name = T::NAME;
            if set.is_empty() {
                f.write_fmt(format_args!("    export namespace {name} {{}}\n"))?;
            } else {
                f.write_fmt(format_args!("    export namespace {name} {{\n"))?;
                for export in set.values() {
                    f.write_str("        ")?;
                    Display::fmt(&TsFormatter(export), f)?;
                    f.write_str("\n")?;
                    f.write_str("        ")?;
                    Display::fmt(&ZodFormatter(export), f)?;
                    f.write_str("\n")?;
                }

                f.write_str("    }\n")?;
            }
            std::fmt::Result::Ok(())
        }

        fmt_part(f, &self.input)?;
        fmt_part(f, &self.output)?;
        fmt_part(f, &self.io)?;

        Ok(())
    }
}

// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
//
//

#[cfg(test)]
mod test {

    #![allow(dead_code)]
    use crate::{
        test_utils::make_args,
        Kind::{Input, Output},
    };

    use super::*;

    use pretty_assertions::assert_eq;

    use z::*;

    struct Generic<T> {
        inner: T,
    }

    struct Ns;
    struct Ns2;

    impl Namespace for Ns {
        const NAME: &'static str = "Ns";
    }

    impl Namespace for Ns2 {
        const NAME: &'static str = "Ns2";
    }

    impl<T> Type<Input> for Generic<T>
    where
        T: Type<Input>,
    {
        type Ns = Ns;
        const NAME: &'static str = "Generic";
        const INLINE: bool = false;

        fn value() -> ZodType<Input> {
            ZodObject::builder()
                .fields(vec![ZodNamedField::builder()
                    .name("inner")
                    .value(ZodTypeInner::Generic(String::from("T")))
                    .build()])
                .build()
                .into()
        }

        fn args() -> Vec<GenericArgument<Kind::Input>> {
            make_args!(T)
        }

        fn visit_dependencies(visitor: &mut DependencyVisitor<Input>) {
            T::visit_dependencies(visitor)
        }
    }

    impl<T> Type<Output> for Generic<T>
    where
        T: Type<Output>,
    {
        type Ns = Ns;
        const NAME: &'static str = "Generic";
        const INLINE: bool = false;

        fn value() -> ZodType<Output> {
            ZodObject::builder()
                .fields(vec![ZodNamedField::builder()
                    .name("inner")
                    .value(ZodTypeInner::Generic(String::from("T")))
                    .build()])
                .build()
                .into()
        }

        fn args() -> Vec<GenericArgument<Kind::Output>> {
            make_args!(T)
        }

        fn visit_dependencies(visitor: &mut DependencyVisitor<Output>) {
            T::visit_dependencies(visitor)
        }
    }

    struct Alias;

    impl Type<Kind::Input> for Alias {
        type Ns = Ns;
        const NAME: &'static str = "Alias";
        const INLINE: bool = false;

        fn value() -> ZodType<Kind::Input> {
            u8::inline().into()
        }
        fn visit_dependencies(visitor: &mut DependencyVisitor<Kind::Input>) {
            u8::visit_dependencies(visitor)
        }
        fn args() -> Vec<GenericArgument<Kind::Input>> {
            Vec::new()
        }
    }

    impl Type<Kind::Output> for Alias {
        type Ns = Ns;
        const NAME: &'static str = "Alias";
        const INLINE: bool = false;

        fn value() -> ZodType<Kind::Output> {
            String::inline().into()
        }
        fn visit_dependencies(visitor: &mut DependencyVisitor<Kind::Output>) {
            String::visit_dependencies(visitor)
        }
        fn args() -> Vec<GenericArgument<Kind::Output>> {
            Vec::new()
        }
    }

    struct Nested<T> {
        inner: Generic<T>,
    }

    impl<T: Type<Input>> Type<Input> for Nested<T> {
        type Ns = Ns;
        const NAME: &'static str = "Nested";
        const INLINE: bool = false;

        fn value() -> ZodType<Input> {
            ZodObject::builder()
                .fields(vec![ZodNamedField::builder()
                    .name("inner")
                    .value(<Generic<crate::test_utils::const_str!('T')> as TypeExt<
                        Input,
                    >>::inline())
                    .build()])
                .build()
                .into()
        }

        fn args() -> Vec<GenericArgument<Kind::Input>> {
            make_args!(T)
        }

        fn visit_dependencies(visitor: &mut DependencyVisitor<Input>) {
            T::visit_dependencies(visitor)
        }
    }

    struct OutputOnly;

    impl Type<Kind::Output> for OutputOnly {
        type Ns = Ns;
        const NAME: &'static str = "OutputOnly";
        const INLINE: bool = false;

        fn value() -> ZodType<Kind::Output> {
            String::inline().into()
        }

        fn visit_dependencies(visitor: &mut DependencyVisitor<Kind::Output>) {
            String::visit_dependencies(visitor)
        }
        fn args() -> Vec<GenericArgument<Kind::Output>> {
            Vec::new()
        }
    }

    #[test]
    fn inline_transparent_ok() {
        assert_eq!(
            TsFormatter(&<Alias as TypeExt<Input>>::export().unwrap()).to_string(),
            "export type Alias = Rs.input.U8;"
        );

        assert_eq!(
            TsFormatter(&<Alias as TypeExt<Output>>::export().unwrap()).to_string(),
            "export type Alias = Rs.output.String;"
        );
    }

    #[test]
    fn ok1() {
        assert_eq!(
            TsFormatter(&<Generic::<Alias> as TypeExt<Kind::Output>>::inline()).to_string(),
            "Ns.output.Generic<Ns.output.Alias>"
        );
        assert_eq!(
            TsFormatter(&<Generic::<Alias> as TypeExt<Kind::Input>>::inline()).to_string(),
            "Ns.input.Generic<Ns.input.Alias>"
        );
    }

    #[test]
    fn export_map_ok() {
        let map = ExportMap::new(
            [
                Export::builder()
                    .name("hello")
                    .ns(Ns::NAME)
                    .value(ZodTypeInner::Generic(String::from("MyGeneric")))
                    .build(),
                Export::builder()
                    .name("world")
                    .ns(Ns2::NAME)
                    .value(
                        ZodObject::builder()
                            .fields(vec![ZodNamedField::builder()
                                .name("hello")
                                .value(Reference::builder().name("hello").ns(Ns::NAME).build())
                                .build()])
                            .build(),
                    )
                    .build(),
            ],
            [Export::builder()
                .name("hello")
                .ns(Ns::NAME)
                .value(ZodTypeInner::Generic(String::from("MyGeneric")))
                .build()],
        );

        assert_eq!(
            map.to_string().trim(),
            r#"
export namespace Ns {
    export namespace input {
        export type hello = Ns.io.hello;
        export const hello = Ns.io.hello;
    }
    export namespace output {
        export type hello = Ns.io.hello;
        export const hello = Ns.io.hello;
    }
    export namespace io {
        export type hello = MyGeneric;
        export const hello = MyGeneric;
    }
}
export namespace Ns2 {
    export namespace input {
        export interface world { hello: Ns.input.hello }
        export const world = z.object({ hello: Ns.input.hello });
    }
    export namespace output {}
    export namespace io {}
}"#
            .trim()
        );
    }
}
