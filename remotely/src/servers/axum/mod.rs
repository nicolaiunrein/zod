mod subscription;

use axum::{
    body::{boxed, Body, BoxBody},
    http,
    http::{HeaderValue, Response},
    response::IntoResponse,
};

pub struct RpcResponse(remotely_core::Response);

impl IntoResponse for RpcResponse {
    fn into_response(self) -> Response<BoxBody> {
        let body: Body = serde_json::to_string(&self.0).unwrap().into();
        let mut resp = Response::new(boxed(body));
        resp.headers_mut().insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        resp
    }
}
