use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::utils::Separated;

use super::{Ts, Zod, ZodType, ZodTypeInner};

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodTuple<Io> {
    #[builder(default)]
    pub fields: Vec<ZodType<Io>>,
}

impl<Io> Display for Zod<'_, ZodTuple<Io>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.fields.iter().map(Zod).collect::<Vec<_>>();
        f.write_fmt(format_args!("z.tuple([{}])", Separated(", ", &variants)))
    }
}

impl<Io> Display for Ts<'_, ZodTuple<Io>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.fields.iter().map(Ts).collect::<Vec<_>>();
        f.write_fmt(format_args!("[{}]", Separated(", ", &variants)))
    }
}

impl<Io> From<ZodTuple<Io>> for ZodTypeInner<Io> {
    fn from(value: ZodTuple<Io>) -> Self {
        ZodTypeInner::Tuple(value)
    }
}

#[cfg(test)]
mod test {
    use crate::types::{ZodNumber, ZodString};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fmt_ok() {
        assert_eq!(
            Zod(&ZodTuple::builder()
                .fields(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "z.tuple([z.string(), z.number()])"
        );

        assert_eq!(
            Ts(&ZodTuple::builder()
                .fields(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "[string, number]"
        );
    }
}
