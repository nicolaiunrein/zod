mod namespace;

pub use namespace::*;

pub type ResponseSender = futures::channel::mpsc::UnboundedSender<serde_json::Value>;

use std::{collections::BTreeMap, path::Path};

type FileMap = BTreeMap<&'static Path, String>;

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

    async fn handle_request(&mut self, req: Request, res: ResponseSender);
}

pub trait ClientCodegen {
    fn get() -> String;
}

#[derive(serde::Deserialize)]
pub enum Request {
    Method(serde_json::Value),
}

#[derive(thiserror::Error, Debug, serde::Serialize)]
pub enum Error {
    #[error("JsonError: {0}")]
    #[serde(serialize_with = "ser_display")]
    #[serde(rename = "JsonError")]
    Json(#[from] serde_json::Error),
}

fn ser_display<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: std::fmt::Display,
    S: serde::Serializer,
{
    serializer.collect_str(value)
}
