//! Types used to build the RPC server/client and messages
#![deny(unsafe_code)]

mod error;
pub mod server;

pub use error::*;

/// The sending half of a Response channel
pub type ResponseSender = futures::channel::mpsc::UnboundedSender<Response>;

/// The json representation of a RPC Request
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Request {
    Exec {
        id: usize,
        #[serde(flatten)]
        value: serde_json::Value,
    },
    CancelStream {
        id: usize,
    },
}

/// The json representation of a RPC Response
#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Response {
    Method {
        id: usize,
        data: serde_json::Value,
    },
    Stream {
        id: usize,
        data: serde_json::Value,
    },
    Error {
        id: Option<usize>,
        data: error::Error,
    },
}

impl Response {
    pub fn error(id: impl Into<Option<usize>>, err: impl Into<error::Error>) -> Self {
        Self::Error {
            id: id.into(),
            data: err.into(),
        }
    }

    pub fn method(id: usize, value: impl serde::ser::Serialize) -> Self {
        match serde_json::to_value(value) {
            Ok(data) => Self::Method { id, data },
            Err(data) => Self::error(id, data),
        }
    }

    pub fn stream(id: usize, value: impl serde::ser::Serialize) -> Self {
        match serde_json::to_value(value) {
            Ok(data) => Self::Stream { id, data },
            Err(value) => Self::error(id, value),
        }
    }
}
