use colored::Colorize;
use quote::quote;
use quote::ToTokens;

pub trait TokenStreamExt {
    fn to_formatted_string(&self) -> Result<String, syn::Error>;
}

impl<T> TokenStreamExt for T
where
    T: ToTokens,
{
    fn to_formatted_string(&self) -> Result<String, syn::Error> {
        formatted(self)
    }
}

pub(crate) fn formatted(input: impl ToTokens) -> Result<String, syn::Error> {
    let file = quote!(fn test() {#input});
    match syn::parse_file(&file.to_string()) {
        Ok(syntax_tree) => Ok(prettyplease::unparse(&syntax_tree)),
        Err(err) => {
            let start = err.span().start();
            let end = err.span().end();
            let text = file
                .to_string()
                .lines()
                .enumerate()
                .take_while(|(index, _)| *index + 1 >= start.line && *index + 1 <= end.line)
                .map(|(_, line)| line)
                .collect::<Vec<_>>()
                .join("\n");

            let range = 100;

            let slice = {
                let pre_start = start.column.saturating_sub(range);
                let c_count = text.chars().count();
                if end.column + range >= c_count {
                    [
                        text[pre_start..start.column].blue().to_string(),
                        text[start.column..end.column].red().italic().to_string(),
                        text[end.column..].blue().to_string(),
                    ]
                    .join("")
                } else {
                    [
                        text[pre_start..start.column].blue().to_string(),
                        text[start.column..end.column].red().italic().to_string(),
                        text[end.column..(end.column + range)].blue().to_string(),
                    ]
                    .join("")
                }
            };

            println!("\n{slice}\n");
            Err(err)
        }
    }
}

#[cfg(test)]
macro_rules! const_str {
    ($first: tt, $($rest: tt),*) => {
        $crate::utils::ConstStr::<$first, crate::test_utils::const_str!($($rest),*)>
    };

    ($first: tt) => {
        $crate::utils::ConstStr::<$first, $crate::utils::End>
    };
}

#[cfg(test)]
pub(crate) use const_str;

#[cfg(test)]
macro_rules! make_args {
    ($($ident: ident),*) => {
        ::std::vec![$($crate::GenericArgument::new::<$ident>(stringify!($ident))),*]
    }
}

#[cfg(test)]
pub(crate) use make_args;
