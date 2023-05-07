use serde_json::json;
use zod::Namespace;

#[derive(Namespace)]
struct Ns;

#[zod::rpc]
impl Ns {
    async fn no_args(&mut self) {}
    async fn one_arg(&mut self, _a: u8) {}
}

#[test]
fn no_args_ok() {
    serde_json::from_value::<<Ns as zod::core::rpc::RpcNamespace>::Req>(
        json!({"method": "no_args", "args": []}),
    )
    .unwrap();
}

#[test]
fn one_arg_ok() {
    serde_json::from_value::<<Ns as zod::core::rpc::RpcNamespace>::Req>(
        json!({"method": "one_arg", "args": [1]}),
    )
    .unwrap();
}
