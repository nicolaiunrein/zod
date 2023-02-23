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
        schema: &<MyEntity as remotely_zod::Codegen>::schema,
        type_def: &<MyEntity as remotely_zod::Codegen>::type_def,
    }
);

inventory::submit!(
    remotely::__private::codegen::namespace::NsMember::Interface {
        ns_name: "Pixera",
        name: "MyEntity2",
        schema: &<MyEntity2 as remotely_zod::Codegen>::schema,
        type_def: &<MyEntity2 as remotely_zod::Codegen>::type_def,
    }
);

impl remotely::__private::codegen::namespace::Namespace for Watchout {
    const NAME: &'static str = "Watchout";
    type Req = WatchoutReq;
}

impl remotely::__private::codegen::namespace::Namespace for Pixera {
    const NAME: &'static str = "Pixera";
    type Req = PixeraReq;
}

inventory::submit!(remotely::__private::codegen::namespace::NsMember::Method {
    ns_name: "Watchout",
    name: "hello",
    args: &|| vec![
        (
            "s",
            <String as remotely_zod::Codegen>::type_def(),
            <String as remotely_zod::Codegen>::schema()
        ),
        (
            "num",
            <usize as remotely_zod::Codegen>::type_def(),
            <usize as remotely_zod::Codegen>::schema()
        )
    ],

    res: &<usize as remotely_zod::Codegen>::type_def,
});

inventory::submit!(remotely::__private::codegen::namespace::NsMember::Method {
    ns_name: "Watchout",
    name: "nested",
    args: &|| vec![(
        "value",
        <MyEntity as remotely_zod::Codegen>::type_def(),
        <MyEntity as remotely_zod::Codegen>::schema()
    )],

    res: &<usize as remotely_zod::Codegen>::type_def,
});

inventory::submit!(remotely::__private::codegen::namespace::NsMember::Stream {
    ns_name: "Watchout",
    name: "hello_stream",
    args: &|| vec![(
        "num",
        <usize as remotely_zod::Codegen>::type_def(),
        <usize as remotely_zod::Codegen>::schema()
    )],
    res: &<usize as remotely_zod::Codegen>::type_def,
});

#[async_trait::async_trait]
impl remotely::__private::server::Backend for MyBackend {
    fn generate<T>() -> String
    where
        T: remotely::__private::codegen::ClientCodegen,
    {
        let mut code = T::get();
        // repeat for all namespaces
        code.push_str(&<Watchout as remotely::__private::codegen::namespace::Namespace>::code());
        code.push_str(&<Pixera as remotely::__private::codegen::namespace::Namespace>::code());
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
    Pixera(<Pixera as remotely::__private::codegen::namespace::Namespace>::Req),
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "method")]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub enum PixeraReq {}

impl MyBackendReq {
    async fn call(
        self,
        id: usize,
        backend: &mut MyBackend,
        sender: remotely::__private::ResponseSender,
    ) -> Option<tokio::task::JoinHandle<()>> {
        match self {
            MyBackendReq::Watchout(req) => req.call(id, &mut backend.0, sender).await,
            MyBackendReq::Pixera(req) => req.call(id, &mut backend.1, sender).await,
        }
    }
}

impl PixeraReq {
    async fn call(
        self,
        _id: usize,
        _ctx: &mut Pixera,
        _sender: remotely::__private::ResponseSender,
    ) -> Option<tokio::task::JoinHandle<()>> {
        None
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
