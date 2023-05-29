//! ## Problem :
//! The impl is split between Argument types and Response types.
//! The Traits are the same. Find a way to uniform them.
//!
//!
//! ## Solution:
//! Remove the Role::Io and IoType alltogether.
//! The implementor should not care.
//! We require InputType to be implemented for RPC arguments and OutputType for Rpc response
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
//!
mod build_ins;
pub mod const_str;
pub mod derive_internals;
pub mod types;
mod utils;

#[cfg(test)]
pub mod test_utils;

use std::{
    collections::{BTreeMap, HashSet},
    fmt::Display,
    marker::PhantomData,
};

use build_ins::Rs;
use typed_builder::TypedBuilder;
use types::{Ts, Zod, ZodExport, ZodType, ZodTypeInner};

// TODO: rename to Kind
pub mod kind {
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

// TODO: seal this trait
pub trait IoKind {
    const NAME: &'static str;
}

impl IoKind for kind::Input {
    const NAME: &'static str = "input";
}
impl IoKind for kind::Output {
    const NAME: &'static str = "output";
}
impl IoKind for kind::EitherIo {
    const NAME: &'static str = "io";
}

pub struct DependencyVisitor<Io> {
    exports: HashSet<ZodExport<Io>>,
}

impl<Io> DependencyVisitor<Io> {
    pub fn visit<T>(&mut self)
    where
        T: Type<Io>,
        Io: std::hash::Hash + Eq,
        Io: Clone,
    {
        self.exports.insert(T::export());
    }
}

#[macro_export]
macro_rules! make_args {
    ($($ident: ident),+) => {
        vec![$((stringify!($ident), $ident::get_ref().into()))+]
    }
}

pub trait Type<Io>
where
    Io: Clone,
{
    type Ns: Namespace;
    const NAME: &'static str;

    fn value() -> ZodType<Io>;

    fn args() -> Vec<(&'static str, ZodType<Io>)> {
        Vec::new()
    }

    // TODO: make required
    fn visit_dependencies(_visitor: &mut DependencyVisitor<Io>) {}

    fn export() -> ZodExport<Io> {
        ZodExport {
            name: String::from(Self::NAME),
            ns: String::from(Self::Ns::NAME),
            args: Self::args()
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>(),

            value: Self::value(),
        }
    }

    fn get_ref() -> Reference<Io> {
        let export = Self::export();
        Reference {
            name: export.name,
            ns: export.ns,
            args: Self::args()
                .iter()
                .map(|(_, ty)| ZodType::clone(ty))
                .collect(),
            _phantom: Default::default(),
        }
    }
}

pub trait Namespace {
    const NAME: &'static str;
}

impl<const C: char, T: const_str::Chain> Type<kind::Input> for const_str::ConstStr<C, T> {
    type Ns = Rs;
    const NAME: &'static str = "";

    fn get_ref() -> Reference<kind::Input> {
        todo!()
    }
    fn value() -> ZodType<kind::Input> {
        panic!("todo... not supported")
    }
}

impl<const C: char, T: const_str::Chain> Type<kind::Output> for const_str::ConstStr<C, T> {
    type Ns = Rs;
    const NAME: &'static str = "";

    fn get_ref() -> Reference<kind::Output> {
        todo!()
    }
    fn value() -> ZodType<kind::Output> {
        panic!("todo... not supported")
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

    #[builder(default)]
    _phantom: PhantomData<Io>,
}

/// Type alias into the common namespace. The fields are pub(crate) to ensure this type is only
/// constructed when processing the export map
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct Alias {
    pub(crate) name: String,
    pub(crate) ns: String,
}

impl<'a> Display for Zod<'a, Alias> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}.{}.{}",
            self.ns,
            kind::EitherIo::NAME,
            self.name
        ))
    }
}

impl<'a> Display for Ts<'a, Alias> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}.{}.{}",
            self.ns,
            kind::EitherIo::NAME,
            self.name
        ))
    }
}

impl<'a, Io> Display for Ts<'a, Reference<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}", self.0.ns, Io::NAME, self.0.name))?;
        if !self.0.args.is_empty() {
            let args = self.0.args.iter().map(Ts).collect::<Vec<_>>();

            f.write_fmt(format_args!("<{}>", utils::Separated(", ", &args)))?;
        }
        Ok(())
    }
}

impl<'a, Io> Display for Zod<'a, Reference<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}", self.0.ns, Io::NAME, self.0.name))?;
        if !self.0.args.is_empty() {
            let args = self.0.args.iter().map(Zod).collect::<Vec<_>>();
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

impl From<Reference<kind::Input>> for Reference<kind::EitherIo> {
    fn from(other: Reference<kind::Input>) -> Self {
        Self {
            name: other.name,
            ns: other.ns,
            args: other.args.into_iter().map(|arg| arg.into()).collect(),
            _phantom: PhantomData,
        }
    }
}

impl From<Reference<kind::Output>> for Reference<kind::EitherIo> {
    fn from(other: Reference<kind::Output>) -> Self {
        Self {
            name: other.name,
            ns: other.ns,
            args: other.args.into_iter().map(|arg| arg.into()).collect(),
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
            _phantom,
        } = self;

        name == &other.name && ns == &other.ns && args == &other.args
    }
}

// pub fn collect_input_exports<T: InputType>() -> HashSet<ZodExport> {
//     let mut set = HashSet::new();
//     T::visit_input_exports(&mut set);
//     set
// }
//
// pub fn collect_output_exports<T: OutputType>() -> HashSet<ZodExport> {
//     let mut set = HashSet::new();
//     T::visit_output_exports(&mut set);
//     set
// }
//
macro_rules! make_eq {
    ($name: ident { $($fields: ident),* }) => {
        impl<A, B> PartialEq<$name<A>> for $name<B> {
            fn eq(&self, other: &$name<A>) -> bool {
                let Self {
                    $($fields),*
                } = self;

                // $(
                //     if $fields != &other.$fields {
                //         return false
                //     }
                // )*
                // true
                $($fields == &other.$fields)&&*
            }
        }
    }
}
pub(crate) use make_eq;

struct NsMap {
    input: BTreeMap<String, ZodExport<kind::Input>>,
    output: BTreeMap<String, ZodExport<kind::Output>>,
    io: BTreeMap<String, ZodExport<kind::EitherIo>>,
}

impl NsMap {
    fn insert_input(&mut self, name: String, mut input: ZodExport<kind::Input>) {
        if let Some(output) = self.output.get_mut(&name) {
            if &mut input == output {
                let merged = ZodExport::<kind::EitherIo>::from(input.clone());

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

    fn insert_output(&mut self, name: String, mut output: ZodExport<kind::Output>) {
        if let Some(input) = self.input.get_mut(&name) {
            if &mut output == input {
                let merged = ZodExport::<kind::EitherIo>::from(output.clone());

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
        input_exports: impl IntoIterator<Item = ZodExport<kind::Input>>,
        output_exports: impl IntoIterator<Item = ZodExport<kind::Output>>,
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
            set: &BTreeMap<String, ZodExport<T>>,
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
                    Display::fmt(&Ts(export), f)?;
                    f.write_str("\n")?;
                    f.write_str("        ")?;
                    Display::fmt(&Zod(export), f)?;
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
    use crate::kind::{Input, Output};

    use super::*;

    use pretty_assertions::assert_eq;

    use types::*;

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

        fn value() -> ZodType<Input> {
            ZodObject::builder()
                .fields(vec![ZodNamedField::builder()
                    .name("inner")
                    .value(ZodTypeInner::Generic(String::from("T")))
                    .build()])
                .build()
                .into()
        }

        fn args() -> Vec<(&'static str, ZodType<Input>)> {
            vec![("T", T::get_ref().into())] //todo
        }
    }

    impl<T> Type<Output> for Generic<T>
    where
        T: Type<Output>,
    {
        type Ns = Ns;
        const NAME: &'static str = "Generic";

        fn value() -> ZodType<Output> {
            ZodObject::builder()
                .fields(vec![ZodNamedField::builder()
                    .name("inner")
                    .value(ZodTypeInner::Generic(String::from("T")))
                    .build()])
                .build()
                .into()
        }

        fn args() -> Vec<(&'static str, ZodType<Output>)> {
            vec![("T", T::get_ref().into())]
        }
    }

    struct Alias;

    impl Type<kind::Input> for Alias {
        type Ns = Ns;
        const NAME: &'static str = "Alias";

        fn value() -> ZodType<kind::Input> {
            u8::get_ref().into()
        }
    }

    impl Type<kind::Output> for Alias {
        type Ns = Ns;
        const NAME: &'static str = "Alias";

        fn value() -> ZodType<kind::Output> {
            String::get_ref().into()
        }
    }

    struct Nested<T> {
        inner: Generic<T>,
    }

    impl<T: Type<Input>> Type<Input> for Nested<T> {
        type Ns = Ns;
        const NAME: &'static str = "Nested";

        fn value() -> ZodType<Input> {
            ZodObject::builder()
                .fields(vec![ZodNamedField::builder()
                    .name("inner")
                    .value(<Generic<crate::const_str!('T')> as Type<Input>>::get_ref())
                    .build()])
                .build()
                .into()
        }

        fn args() -> Vec<(&'static str, ZodType<Input>)> {
            make_args!(T)
        }
    }

    struct OutputOnly;

    impl Type<kind::Output> for OutputOnly {
        type Ns = Ns;
        const NAME: &'static str = "OutputOnly";

        fn value() -> ZodType<kind::Output> {
            String::get_ref().into()
        }
    }

    #[test]
    fn inline_transparent_ok() {
        assert_eq!(
            Ts(&<Alias as Type<Input>>::export()).to_string(),
            "export type Alias = Rs.input.U8;"
        );

        assert_eq!(
            Ts(&<Alias as Type<Output>>::export()).to_string(),
            "export type Alias = Rs.output.String;"
        );
    }

    #[test]
    fn ok1() {
        assert_eq!(
            Ts(&<Generic::<Alias> as Type<kind::Output>>::get_ref()).to_string(),
            "Ns.output.Generic<Ns.output.Alias>"
        );
        assert_eq!(
            Ts(&<Generic::<Alias> as Type<kind::Input>>::get_ref()).to_string(),
            "Ns.input.Generic<Ns.input.Alias>"
        );
    }

    #[test]
    fn export_map_ok() {
        let map = ExportMap::new(
            [
                ZodExport::builder()
                    .name("hello")
                    .ns(Ns::NAME)
                    .value(ZodTypeInner::Generic(String::from("MyGeneric")))
                    .build(),
                ZodExport::builder()
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
            [ZodExport::builder()
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
