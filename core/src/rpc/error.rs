/// Represent RPC Server Errors to be sent to the client
#[derive(thiserror::Error, Debug, serde::Serialize)]
#[serde(tag = "name", content = "message")]
pub enum Error {
    #[error("JsonError: {0}")]
    #[serde(serialize_with = "ser_display")]
    #[serde(rename = "JsonError")]
    Json(#[from] serde_json::Error),

    #[error("Unknown Namespace: {0}")]
    UnknownNamespace(String),
}

fn ser_display<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: std::fmt::Display,
    S: serde::Serializer,
{
    serializer.collect_str(value)
}
