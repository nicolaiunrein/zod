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
