use remotely_core::codegen::ClientCodegen;

pub struct WebsocketClient;

impl ClientCodegen for WebsocketClient {
    fn get() -> String {
        String::from(include_str!("./static/websocket_client.ts"))
    }
}
