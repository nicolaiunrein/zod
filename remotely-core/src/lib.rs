mod namespace;

pub use namespace::*;

pub type ResponseSender = futures::channel::mpsc::UnboundedSender<Response>;
pub type SubscriberMap = HashMap<usize, tokio::task::JoinHandle<()>>;

use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

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

    async fn handle_request(
        &mut self,
        req: Request,
        res: ResponseSender,
        subscribers: &mut SubscriberMap,
    );
}

pub trait ClientCodegen {
    fn get() -> String;
}

#[derive(serde::Deserialize, Debug)]
pub enum Request {
    Request { id: usize, value: serde_json::Value },
    StreamCancel { id: usize },
}

#[derive(serde::Serialize, Debug)]
pub enum Response {
    Method { id: usize, value: serde_json::Value },
    Stream { id: usize, event: serde_json::Value },
    Error { id: usize, value: Error },
}

impl Response {
    pub fn error(id: usize, err: impl Into<Error>) -> Self {
        Self::Error {
            id,
            value: err.into(),
        }
    }

    pub fn method(id: usize, value: impl serde::ser::Serialize) -> Self {
        match serde_json::to_value(value) {
            Ok(value) => Self::Method { id, value },
            Err(value) => Self::error(id, value),
        }
    }

    pub fn stream(id: usize, value: impl serde::ser::Serialize) -> Self {
        match serde_json::to_value(value) {
            Ok(event) => Self::Stream { id, event },
            Err(value) => Self::error(id, value),
        }
    }
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
