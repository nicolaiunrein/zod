use std::{collections::BTreeMap, path::Path};

pub mod namespace;

pub trait ClientCodegen {
    fn get() -> String;
}

pub type FileMap = BTreeMap<&'static Path, String>;

#[derive(Debug)]
pub struct FileList(FileMap);

impl FileList {
    #[doc(hidden)]
    pub fn new(inner: FileMap) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> FileMap {
        self.0
    }
}

impl std::ops::Deref for FileList {
    type Target = FileMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
