pub use zod_core::prelude::*;

struct MyNs;

impl Namespace for MyNs {
    const NAME: &'static str = "MyNs";
}

enum SomeEnum<T1, T2> {
    One(T1),
    Two(T2),
}

impl<T1, T2> Type<Kind::Input> for SomeEnum<T1, T2>
where
    T1: Type<Kind::Input>,
    T2: Type<Kind::Input>,
{
    type Ns = MyNs;

    const NAME: &'static str = "SomeOtherStruct";

    const INLINE: bool = false;

    fn value() -> types::ZodType<Kind::Input> {
        types::ZodUnion::builder()
            .variants(vec![T1::inline(), T2::inline()])
            .build()
            .into()
    }

    fn visit_dependencies(visitor: &mut DependencyVisitor<Kind::Input>) {
        T1::visit_dependencies(visitor);
        T2::visit_dependencies(visitor);
    }
    fn args() -> Vec<GenericArgument<Kind::Input>> {
        vec![
            GenericArgument::new::<T1>("T1"),
            GenericArgument::new::<T2>("T2"),
        ]
    }
}

struct SomeOtherStruct<T1, T2> {
    t1: T1,
    t2: SomeEnum<T2, String>,
    value: u8,
}

impl<T1, T2> Type<Kind::Input> for SomeOtherStruct<T1, T2>
where
    T1: Type<Kind::Input>,
    T2: Type<Kind::Input>,
{
    type Ns = MyNs;
    const NAME: &'static str = "MyStruct";
    const INLINE: bool = false;

    fn value() -> types::ZodType<Kind::Input> {
        types::ZodObject::builder()
            .fields(vec![
                types::ZodNamedField::builder()
                    .name("t1")
                    .value(T1::inline())
                    .build(),
                types::ZodNamedField::builder()
                    .name("t2")
                    .value(SomeEnum::<T1, T2>::inline())
                    .build(),
                types::ZodNamedField::builder()
                    .name("value")
                    .value(u8::inline())
                    .build(),
            ])
            .build()
            .into()
    }

    fn visit_dependencies(visitor: &mut DependencyVisitor<Kind::Input>) {
        T1::visit_dependencies(visitor);
        SomeEnum::<T2, String>::visit_dependencies(visitor);
        u8::visit_dependencies(visitor);
    }

    fn args() -> Vec<GenericArgument<Kind::Input>> {
        vec![
            GenericArgument::new::<T1>("T1"),
            GenericArgument::new::<T2>("T2"),
        ]
    }
}
