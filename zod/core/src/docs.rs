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
    let lines = [
        Line::with_link::<()>("()", "https://doc.rust-lang.org/std/primitive.unit.html"),
        Line::new::<bool>(),
        Line::new::<char>(),
        Line::with_link::<Option<T>>("Option<T>", "Option"),
        Line::with_link::<Result<T, E>>("Result<T, E>", "Result"),
        Line::with_link::<Vec<T>>("Vec<T>", "Vec"),
        Line::new::<&'static str>(),
        Line::with_link::<String>("String", None),
        Line::new::<usize>(),
        Line::new::<u8>(),
        Line::new::<u16>(),
        Line::new::<u32>(),
        Line::new::<u64>(),
        Line::new::<u128>(),
        Line::new::<i8>(),
        Line::new::<i16>(),
        Line::new::<i32>(),
        Line::new::<i64>(),
        Line::new::<i128>(),
        Line::new::<usize>(),
        Line::new::<isize>(),
        Line::new::<f32>(),
        Line::new::<f64>(),
        Line::with_link::<Box<T>>("Box<T>", "Box"),
        Line::with_link::<std::sync::Arc<T>>("Arc<T>", "std::sync::Arc"),
        Line::with_link::<std::rc::Rc<T>>("Rc<T>", "std::rc::Rc"),
        Line::with_link::<std::borrow::Cow<'static, T>>("Cow<'static, T>", "std::borrow::Cow"),
        Line::with_link::<std::cell::Cell<T>>("Cell<T>", "std::cell::Cell"),
        Line::with_link::<std::cell::RefCell<T>>("RefCell<T>", "std::cell::RefCell"),
        Line::with_link::<std::sync::Mutex<T>>("Mutex<T>", "std::sync::Mutex"),
        Line::with_link::<std::sync::Weak<T>>("Weak<T>", "std::sync::Weak"),
        Line::with_link::<std::marker::PhantomData<T>>(
            "PhantomData<T>",
            "std::marker::PhantomData",
        ),
        Line::with_link::<std::collections::HashSet<T>>("HashSet<T>", "std::collections::HashSet"),
        Line::with_link::<std::collections::HashMap<T, U>>(
            "HashMap<T1, U2>",
            "std::collections::HashMap",
        ),
        Line::with_link::<std::collections::BTreeSet<T>>(
            "BTreeSet<T>",
            "std::collections::BTreeSet",
        ),
        Line::with_link::<std::collections::BTreeMap<T, U>>(
            "BTreeMap<T1, U2>",
            "std::collections::BTreeMap",
        ),
    ];

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
            "{}\n```ignore// schema\n{}\n```",
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
