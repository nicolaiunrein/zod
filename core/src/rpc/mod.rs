//! Types used to build the RPC server/client and messages
#![deny(unsafe_code)]

mod error;
pub mod server;

pub use error::*;

use crate::types::Usize;
use crate::RequestType;

/// The sending half of a Response channel
pub type ResponseSender = futures::channel::mpsc::UnboundedSender<Response>;

/// The json representation of a RPC Request
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Request {
    Exec {
        id: Usize,
        #[serde(flatten)]
        value: serde_json::Value,
    },
    CancelStream {
        id: Usize,
    },
}

/// The json representation of a RPC Response
#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Response {
    Method {
        id: Usize,
        data: serde_json::Value,
    },
    Stream {
        id: Usize,
        data: serde_json::Value,
    },
    Error {
        id: Option<Usize>,
        data: error::Error,
    },
}

impl Response {
    pub fn error(id: impl Into<Option<Usize>>, err: impl Into<error::Error>) -> Self {
        Self::Error {
            id: id.into(),
            data: err.into(),
        }
    }

    pub fn method(id: usize, value: impl serde::ser::Serialize + RequestType) -> Self {
        match serde_json::to_value(value) {
            Ok(data) => Self::Method {
                id: Usize(id),
                data,
            },
            Err(data) => Self::error(Usize::from(id), data),
        }
    }

    pub fn stream(id: usize, value: impl serde::ser::Serialize + RequestType) -> Self {
        match serde_json::to_value(value) {
            Ok(data) => Self::Stream {
                id: Usize(id),
                data,
            },
            Err(value) => Self::error(Usize(id), value),
        }
    }
}
