use std::fmt;

#[derive(Clone, Debug)]
pub struct Separated<'a, Sep: fmt::Display, Item: fmt::Display + 'a>(pub Sep, pub &'a [Item]);

impl<'a, Sep, Item> fmt::Display for Separated<'a, Sep, Item>
where
    Sep: fmt::Display,
    Item: fmt::Display + 'a,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut iterator = self.1.into_iter();
        if let Some(x) = iterator.next() {
            write!(f, "{}", x)?;
            for item in iterator {
                write!(f, "{}{}", self.0, item)?;
            }
        }
        Ok(())
    }
}
