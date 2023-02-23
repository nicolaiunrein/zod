use super::*;

impl Watchout {
    #[allow(dead_code)]
    #[allow(non_upper_case_globals)]
    const MyEntity: () = ();
}

inventory::submit!(
    remotely::__private::codegen::namespace::NsMember::Interface {
        ns_name: "Watchout",
        name: "MyEntity",
        code: &<MyEntity as ts_rs::TS>::decl,
    }
);

inventory::submit!(
    remotely::__private::codegen::namespace::NsMember::Interface {
        ns_name: "Pixera",
        name: "MyEntity2",
        code: &<MyEntity2 as ts_rs::TS>::decl,
    }
);

impl remotely::__private::codegen::namespace::Namespace for Watchout {
    const NAME: &'static str = "Watchout";
    type Req = WatchoutReq;
}

inventory::submit!(remotely::__private::codegen::namespace::NsMember::Method {
    ns_name: "Watchout",
    name: "hello",
    args: &|| vec![
        ("s", <String as ts_rs::TS>::name()),
        ("num", <usize as ts_rs::TS>::name())
    ],

    res: &<usize as ts_rs::TS>::name,
});

inventory::submit!(remotely::__private::codegen::namespace::NsMember::Stream {
    ns_name: "Watchout",
    name: "hello_stream",
    args: &|| vec![("num", <usize as ts_rs::TS>::name())],
    res: &<() as ts_rs::TS>::name,
});

#[async_trait::async_trait]
impl remotely::__private::server::Backend for MyBackend {
    fn generate<T>() -> String
    where
        T: remotely::__private::codegen::ClientCodegen,
    {
        let mut code = T::get();
        code.push_str(&<Watchout as remotely::__private::codegen::namespace::Namespace>::code());
        // repeat for all namespaces
        code
    }

    async fn handle_request(
        &mut self,
        req: remotely::__private::Request,
        sender: remotely::__private::ResponseSender,
        subscribers: &mut remotely::__private::server::SubscriberMap,
    ) {
        match req {
            remotely::__private::Request::Exec { id, value } => {
                match serde_json::from_value::<MyBackendReq>(value) {
                    Ok(evt) => {
                        if let Some(jh) = evt.call(id, self, sender).await {
                            subscribers.insert(id, jh);
                        }
                    }
                    Err(err) => {
                        let _ = sender
                            .unbounded_send(remotely::__private::Response::error(id, err))
                            .ok();
                    }
                }
            }
            remotely::__private::Request::CancelStream { id } => {
                if let Some(jh) = subscribers.remove(&id) {
                    jh.abort();
                }
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "namespace")]
enum MyBackendReq {
    Watchout(<Watchout as remotely::__private::codegen::namespace::Namespace>::Req),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "method")]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub enum WatchoutReq {
    hello { args: (String, usize) },
    hello_stream { args: (usize,) },
}

impl MyBackendReq {
    async fn call(
        self,
        id: usize,
        backend: &mut MyBackend,
        sender: remotely::__private::ResponseSender,
    ) -> Option<tokio::task::JoinHandle<()>> {
        match self {
            MyBackendReq::Watchout(req) => req.call(id, &mut backend.0, sender).await,
        }
    }
}

impl WatchoutReq {
    async fn call(
        self,
        id: usize,
        ctx: &mut Watchout,
        sender: remotely::__private::ResponseSender,
    ) -> Option<tokio::task::JoinHandle<()>> {
        match self {
            WatchoutReq::hello { args } => {
                let res = ctx.hello(args.0, args.1).await;
                sender
                    .unbounded_send(remotely::__private::Response::method(id, res))
                    .unwrap();
                None
            }
            WatchoutReq::hello_stream { args } => {
                let s = ctx.hello_stream(args.0);
                Some(tokio::spawn(async move {
                    futures::pin_mut!(s);
                    while let Some(evt) = s.next().await {
                        if let Err(err) =
                            sender.unbounded_send(remotely::__private::Response::stream(id, evt))
                        {
                            tracing::warn!(?err, "Failed to emit event");
                            break;
                        }
                    }
                }))
            }
        }
    }
}
