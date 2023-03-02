//! Generate the type-table.md

#[derive(Debug, Clone)]
struct T;

#[derive(Debug, Clone)]
struct U;

impl crate::ZodType for T {
    fn schema() -> String {
        String::from("T")
    }

    fn type_def() -> crate::TsTypeDef {
        crate::TsTypeDef::Type(String::from("T"))
    }
}

impl crate::ZodType for U {
    fn schema() -> String {
        String::from("U")
    }

    fn type_def() -> crate::TsTypeDef {
        crate::TsTypeDef::Type(String::from("U"))
    }
}

type E = U;

pub fn generate() -> String {
    let mut lines = Vec::new();

    lines.push(Line::with_link::<()>(
        "()",
        "https://doc.rust-lang.org/std/primitive.unit.html",
    ));

    lines.push(Line::new::<bool>());
    lines.push(Line::new::<char>());

    lines.push(Line::with_link::<Option<T>>("Option<T>", "Option"));
    lines.push(Line::with_link::<Result<T, E>>("Result<T, E>", "Result"));
    lines.push(Line::with_link::<Vec<T>>("Vec<T>", "Vec"));

    lines.push(Line::new::<&'static str>());
    lines.push(Line::with_link::<String>("String", None));

    lines.push(Line::new::<usize>());
    lines.push(Line::new::<u8>());
    lines.push(Line::new::<u16>());
    lines.push(Line::new::<u32>());
    lines.push(Line::new::<u64>());
    lines.push(Line::new::<u128>());

    lines.push(Line::new::<i8>());
    lines.push(Line::new::<i16>());
    lines.push(Line::new::<i32>());
    lines.push(Line::new::<i64>());
    lines.push(Line::new::<i128>());

    lines.push(Line::new::<usize>());
    lines.push(Line::new::<isize>());

    lines.push(Line::new::<f32>());
    lines.push(Line::new::<f64>());

    lines.push(Line::with_link::<Box<T>>("Box<T>", "Box"));
    lines.push(Line::with_link::<std::sync::Arc<T>>(
        "Arc<T>",
        "std::sync::Arc",
    ));
    lines.push(Line::with_link::<std::rc::Rc<T>>("Rc<T>", "std::rc::Rc"));
    lines.push(Line::with_link::<std::borrow::Cow<'static, T>>(
        "Cow<'static, T>",
        "std::borrow::Cow",
    ));

    lines.push(Line::with_link::<std::cell::Cell<T>>(
        "Cell<T>",
        "std::cell::Cell",
    ));

    lines.push(Line::with_link::<std::cell::RefCell<T>>(
        "RefCell<T>",
        "std::cell::RefCell",
    ));

    lines.push(Line::with_link::<std::sync::Mutex<T>>(
        "Mutex<T>",
        "std::sync::Mutex",
    ));

    lines.push(Line::with_link::<std::sync::Weak<T>>(
        "Weak<T>",
        "std::sync::Weak",
    ));

    lines.push(Line::with_link::<std::marker::PhantomData<T>>(
        "PhantomData<T>",
        "std::marker::PhantomData",
    ));

    lines.push(Line::with_link::<std::collections::HashSet<T>>(
        "HashSet<T>",
        "std::collections::HashSet",
    ));

    lines.push(Line::with_link::<std::collections::HashMap<T, U>>(
        "HashMap<T1, U2>",
        "std::collections::HashMap",
    ));

    lines.push(Line::with_link::<std::collections::BTreeSet<T>>(
        "BTreeSet<T>",
        "std::collections::BTreeSet",
    ));

    lines.push(Line::with_link::<std::collections::BTreeMap<T, U>>(
        "BTreeMap<T1, U2>",
        "std::collections::BTreeMap",
    ));

    let s: String = lines.into_iter().map(|l| l.to_string()).collect();

    format!("## Type Overview\n\n\n\n{s}")
}

struct Line {
    rust: String,
    schema: String,
}

impl Line {
    fn new<T: crate::ZodType>() -> Self {
        let name = std::any::type_name::<T>();
        Self::with_link::<T>(name, name)
    }

    fn with_link<'a, T: crate::ZodType>(name: &'a str, link: impl Into<Option<&'a str>>) -> Self {
        let link = link.into().map(escape);
        let name = escape(name);

        Self {
            rust: match link {
                Some(link) => format!("[{name}]({link})"),
                None => format!("[{name}]"),
            },

            schema: T::schema(),
        }
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}\n```rust,typescript// schema\n{}\n```",
            &self.rust, &self.schema,
        )
    }
}

fn escape(input: &str) -> String {
    // input.to_string()
    input
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
}
