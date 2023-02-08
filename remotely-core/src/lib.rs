mod namespace;

pub use namespace::*;

use std::{collections::BTreeMap, path::Path};

type FileMap = BTreeMap<&'static Path, String>;

// pub type Callback = futures::channel::mpsc::UnboundedSender<serde_json::Value>;

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

#[async_trait::async_trait]
pub trait Backend {
    fn generate<T>() -> FileList
    where
        T: ClientCodegen;

    async fn handle_request(&mut self, req: serde_json::Value) -> serde_json::Value;
}

pub trait ClientCodegen {
    fn get() -> String;
}
