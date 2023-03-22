pub(crate) trait Delimited<F> {
    type Item;
    fn fmt_delimited(
        self,
        f: &mut std::fmt::Formatter<'_>,
        delim: &'static str,
        func: F,
    ) -> std::fmt::Result;

    fn comma_separated(self, f: &mut std::fmt::Formatter<'_>, func: F) -> std::fmt::Result
    where
        Self: Sized,
    {
        self.fmt_delimited(f, ", ", func)
    }
}

impl<Iter, Item, Func> Delimited<Func> for Iter
where
    Iter: Iterator<Item = Item>,
    Func: Fn(&mut std::fmt::Formatter<'_>, Item) -> std::fmt::Result,
{
    type Item = Item;
    fn fmt_delimited(
        self,
        f: &mut std::fmt::Formatter<'_>,
        delim: &'static str,
        func: Func,
    ) -> std::fmt::Result {
        let mut iter = self.peekable();
        while let Some(item) = iter.next() {
            (func)(f, item)?;
            if iter.peek().is_some() {
                f.write_str(delim)?;
            }
        }
        Ok(())
    }
}
