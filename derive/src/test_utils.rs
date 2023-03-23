#[cfg(test)]
use proc_macro2::TokenStream;

#[cfg(test)]
pub(crate) fn compare(a: TokenStream, b: TokenStream) {
    let a = normalize(a.to_string());
    let b = normalize(b.to_string());

    pretty_assertions::assert_eq!(a, b)
}

#[cfg(test)]
fn normalize(input: String) -> String {
    let mut out = String::new();
    let mut iter = input.chars().peekable();

    let mut last: Option<char> = None;

    while let Some(current) = iter.next() {
        if let Some(next) = iter.peek() {
            if current.is_whitespace() && next.is_whitespace() {
                continue;
            }

            if current.is_whitespace() && !next.is_alphanumeric() {
                continue;
            }

            if let Some(last) = last {
                if current.is_whitespace() && !last.is_alphanumeric() {
                    continue;
                }
            }
        }

        out.push(current);
        last = Some(current)
    }
    out
}
