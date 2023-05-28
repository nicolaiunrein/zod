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
mod const_str;
pub mod derive_internals;
pub mod types;
mod utils;

#[cfg(test)]
pub mod test_utils;

use std::{
    collections::{BTreeMap, HashSet},
    fmt::Display,
};

use build_ins::Rs;
use typed_builder::TypedBuilder;
use types::{Role, Ts, Zod, ZodExport, ZodType, ZodTypeInner};

pub trait InputType {
    type Namespace: Namespace;
    fn get_input_ref() -> ZodType;
    fn visit_input_exports(_set: &mut HashSet<ZodExport>);
}

pub trait OutputType {
    type Namespace: Namespace;
    fn get_output_ref() -> ZodType;
    fn visit_output_exports(_set: &mut HashSet<ZodExport>);
}

pub trait Namespace {
    const NAME: &'static str;
}

impl<const C: char, T: const_str::Chain> InputType for const_str::ConstStr<C, T> {
    type Namespace = Rs;
    fn get_input_ref() -> ZodType {
        ZodType::builder()
            .inner(ZodTypeInner::Generic(Self::value().to_string()))
            .build()
    }

    fn visit_input_exports(_set: &mut HashSet<ZodExport>) {}
}

impl<const C: char, T: const_str::Chain> OutputType for const_str::ConstStr<C, T> {
    type Namespace = Rs;
    fn get_output_ref() -> ZodType {
        ZodType::builder()
            .inner(ZodTypeInner::Generic(Self::value().to_string()))
            .build()
    }

    fn visit_output_exports(_set: &mut HashSet<ZodExport>) {}
}

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct Reference {
    #[builder(setter(into))]
    pub name: String,

    #[builder(setter(into))]
    pub ns: String,

    pub role: Role,

    #[builder(default)]
    pub args: Vec<ZodType>,
}

impl From<Reference> for ZodTypeInner {
    fn from(value: Reference) -> Self {
        ZodTypeInner::Reference(value)
    }
}

impl<'a> Display for Ts<'a, Reference> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}", self.0.ns, self.role, self.0.name))?;
        if !self.0.args.is_empty() {
            let args = self.0.args.iter().map(Ts).collect::<Vec<_>>();

            f.write_fmt(format_args!("<{}>", utils::Separated(", ", &args)))?;
        }
        Ok(())
    }
}

impl<'a> Display for Zod<'a, Reference> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}", self.0.ns, self.role, self.0.name))?;
        if !self.0.args.is_empty() {
            let args = self.0.args.iter().map(Zod).collect::<Vec<_>>();
            f.write_fmt(format_args!("({})", utils::Separated(", ", &args)))?;
        }
        Ok(())
    }
}

pub fn collect_input_exports<T: InputType>() -> HashSet<ZodExport> {
    let mut set = HashSet::new();
    T::visit_input_exports(&mut set);
    set
}

pub fn collect_output_exports<T: OutputType>() -> HashSet<ZodExport> {
    let mut set = HashSet::new();
    T::visit_output_exports(&mut set);
    set
}

struct NsMap {
    input: BTreeMap<String, ZodExport>,
    output: BTreeMap<String, ZodExport>,
    io: BTreeMap<String, ZodExport>,
}

pub struct ExportMap(BTreeMap<String, NsMap>);

impl ExportMap {
    pub fn new(exports: impl IntoIterator<Item = ZodExport>) -> Self {
        let mut out = BTreeMap::<String, NsMap>::new();

        for export in exports.into_iter() {
            let ns_map = out.entry(export.ns.clone()).or_insert_with(|| NsMap {
                input: Default::default(),
                output: Default::default(),
                io: Default::default(),
            });

            match export.context {
                Role::InputOnly => ns_map.input.insert(export.name.clone(), export),
                Role::OutputOnly => ns_map.output.insert(export.name.clone(), export),
            };
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
        let mut fmt_part = |name: &'static str, set: &BTreeMap<String, ZodExport>| {
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
        };

        fmt_part("input", &self.input)?;
        fmt_part("output", &self.output)?;
        fmt_part("io", &self.io)?;

        Ok(())
    }
}

// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
//
#[cfg(test)]
mod test {

    #![allow(dead_code)]
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

    impl<T> OutputType for Generic<T>
    where
        T: OutputType,
    {
        type Namespace = Ns;
        fn get_output_ref() -> ZodType {
            Reference::builder()
                .name("Generic")
                .ns(Ns::NAME)
                .role(Role::OutputOnly)
                .args(vec![T::get_output_ref()])
                .build()
                .into()
        }

        fn visit_output_exports(set: &mut HashSet<ZodExport>) {
            let export = ZodExport::builder()
                .ns(Ns::NAME)
                .name("Generic")
                .context(Role::OutputOnly)
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("inner")
                            .value(ZodTypeInner::Generic(String::from("T")))
                            .build()])
                        .build(),
                )
                .build();

            set.insert(export);
            T::visit_output_exports(set)
        }
    }

    impl<T> InputType for Generic<T>
    where
        T: InputType,
    {
        type Namespace = Ns;
        fn get_input_ref() -> ZodType {
            Reference::builder()
                .ns(Ns::NAME)
                .name("Generic")
                .role(Role::InputOnly)
                .args(vec![T::get_input_ref()])
                .build()
                .into()
        }

        fn visit_input_exports(set: &mut HashSet<ZodExport>) {
            let export = ZodExport::builder()
                .ns(Ns::NAME)
                .name("Generic")
                .context(Role::InputOnly)
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("inner")
                            .value(ZodTypeInner::Generic(String::from("T")))
                            .build()])
                        .build(),
                )
                .build();

            set.insert(export);
            T::visit_input_exports(set)
        }
    }

    struct Transparent;

    impl OutputType for Transparent {
        type Namespace = Ns;
        fn get_output_ref() -> ZodType {
            <String as OutputType>::get_output_ref()
        }
        fn visit_output_exports(set: &mut HashSet<ZodExport>) {
            String::visit_output_exports(set);
        }
    }

    impl InputType for Transparent {
        type Namespace = Ns;
        fn get_input_ref() -> ZodType {
            <u8 as InputType>::get_input_ref()
        }
        fn visit_input_exports(set: &mut HashSet<ZodExport>) {
            u8::visit_input_exports(set);
        }
    }

    struct Nested<T> {
        inner: Generic<T>,
    }

    impl<T: OutputType> OutputType for Nested<T> {
        type Namespace = Ns;
        fn get_output_ref() -> ZodType {
            Reference {
                role: Role::OutputOnly,
                name: String::from("Nested"),
                ns: String::from(Ns::NAME),
                args: vec![T::get_output_ref()],
            }
            .into()
        }

        fn visit_output_exports(set: &mut HashSet<ZodExport>) {
            let exp = ZodExport::builder()
                .ns(Ns::NAME)
                .name("Nested")
                .context(Role::OutputOnly)
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("inner")
                            .value(Generic::<crate::const_str!('T')>::get_output_ref())
                            .build()])
                        .build(),
                )
                .build();

            set.insert(exp.into());

            T::visit_output_exports(set)
        }
    }

    impl<T: InputType> InputType for Nested<T> {
        type Namespace = Ns;
        fn get_input_ref() -> ZodType {
            Reference {
                role: Role::InputOnly,
                ns: String::from(Ns::NAME),
                name: String::from("Nested"),
                args: vec![T::get_input_ref()],
            }
            .into()
        }
        fn visit_input_exports(set: &mut HashSet<ZodExport>) {
            let exp = ZodExport::builder()
                .ns(Ns::NAME)
                .name("Nested")
                .context(Role::InputOnly)
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("inner")
                            .value(Generic::<crate::const_str!('T')>::get_input_ref())
                            .build()])
                        .build(),
                )
                .build();

            set.insert(exp.into());

            T::visit_input_exports(set)
        }
    }

    struct OutputOnly;

    impl OutputType for OutputOnly {
        type Namespace = Ns;
        fn get_output_ref() -> ZodType {
            Reference {
                role: Role::OutputOnly,
                ns: String::from(Ns::NAME),
                name: String::from("OutputOnly"),
                args: Vec::new(),
            }
            .into()
        }
        fn visit_output_exports(set: &mut HashSet<ZodExport>) {
            set.insert(
                ZodExport::builder()
                    .ns(Ns::NAME)
                    .name("OutputOnly")
                    .context(Role::OutputOnly)
                    .value(String::get_output_ref())
                    .build(),
            );
        }
    }

    #[test]
    fn inline_transparent_ok() {
        assert_eq!(
            Ts(&Transparent::get_output_ref()).to_string(),
            "Rs.io.String"
        );
        assert_eq!(Ts(&Transparent::get_input_ref()).to_string(), "Rs.io.U8");
    }

    #[test]
    fn debug() {
        let u8_export = collect_input_exports::<u8>().into_iter().next().unwrap();
        let string_export = collect_input_exports::<String>()
            .into_iter()
            .next()
            .unwrap();

        let generic_input_export = ZodExport::builder()
            .ns(Ns::NAME)
            .name("Generic")
            .context(Role::InputOnly)
            .args(&["T"])
            .value(
                ZodObject::builder()
                    .fields(vec![ZodNamedField::builder()
                        .name("inner")
                        .value(ZodTypeInner::Generic(String::from("T")))
                        .build()])
                    .build(),
            )
            .build();

        let generic_output_export = {
            ZodExport {
                context: Role::OutputOnly,
                ..generic_input_export.clone()
            }
        };

        let output_only_export = ZodExport::builder()
            .ns(Ns::NAME)
            .name("OutputOnly")
            .context(Role::OutputOnly)
            .value(String::get_output_ref())
            .build();

        assert_eq!(
            Ts(&Generic::<Transparent>::get_output_ref()).to_string(),
            "Ns.output.Generic<Rs.io.String>"
        );

        assert_eq!(
            Ts(&Generic::<crate::const_str!('M', 'Y', '_', 'T')>::get_output_ref()).to_string(),
            "Ns.output.Generic<MY_T>"
        );

        assert_eq!(
            Ts(&Generic::<Transparent>::get_input_ref()).to_string(),
            "Ns.input.Generic<Rs.io.U8>"
        );

        assert_eq!(
            collect_output_exports::<Generic::<u8>>(),
            [u8_export.clone(), generic_output_export.clone()]
                .into_iter()
                .collect()
        );

        assert_eq!(
            collect_output_exports::<Transparent>(),
            [string_export.clone()].into_iter().collect()
        );

        assert_eq!(
            collect_output_exports::<Generic::<Transparent>>(),
            [string_export.clone(), generic_output_export.clone()]
                .into_iter()
                .collect()
        );

        assert_eq!(
            collect_output_exports::<Generic::<OutputOnly>>(),
            [generic_output_export.clone(), output_only_export.clone()]
                .into_iter()
                .collect()
        );

        assert_eq!(
            <Generic::<OutputOnly>>::get_output_ref(),
            Reference {
                role: Role::OutputOnly,
                ns: String::from(Ns::NAME),
                name: String::from("Generic"),
                args: vec![Reference {
                    role: Role::OutputOnly,
                    ns: String::from(Ns::NAME),
                    name: String::from("OutputOnly"),
                    args: vec![]
                }
                .into()]
            }
            .into()
        );
    }

    #[test]
    fn reference_context() {
        assert_eq!(
            collect_output_exports::<OutputOnly>()
                .into_iter()
                .map(|exp| Zod(&exp).to_string())
                .collect::<HashSet<_>>(),
            [String::from("export const OutputOnly = Rs.io.String;")]
                .into_iter()
                .collect()
        )
    }

    #[test]
    fn export_map_ok() {
        let map = ExportMap::new([
            ZodExport::builder()
                .name("hello")
                .ns(Ns::NAME)
                .context(Role::InputOnly)
                .value(ZodTypeInner::Generic(String::from("MyGeneric")))
                .build(),
            ZodExport::builder()
                .name("world")
                .ns(Ns2::NAME)
                .context(Role::InputOnly)
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("hello")
                            .value(
                                Reference::builder()
                                    .name("hello")
                                    .ns(Ns::NAME)
                                    .role(Role::InputOnly)
                                    .build(),
                            )
                            .build()])
                        .build(),
                )
                .build(),
        ]);

        assert_eq!(
            map.to_string().trim(),
            r#"
export namespace Ns {
    export namespace input {}
    export namespace output {}
    export namespace io {
        export type hello = MyGeneric;
        export const hello = MyGeneric;
    }
}
export namespace Ns2 {
    export namespace input {
        export interface world { hello: Ns.io.hello }
        export const world = z.object({ hello: Ns.io.hello });
    }
    export namespace output {}
    export namespace io {}
}"#
            .trim()
        );
    }
}
