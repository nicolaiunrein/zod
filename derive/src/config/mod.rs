mod container;
mod field;

pub(crate) use container::ContainerConfig;
pub(crate) use container::TagType;
pub(crate) use field::FieldConfig;

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(test, derive(Default))]
pub(crate) enum Derive {
    #[cfg_attr(test, default)]
    Request,
    Response,
}
