use remotely::__private::{Request, ResponseSender};

use super::*;

impl Watchout {
    #[allow(dead_code)]
    #[allow(non_upper_case_globals)]
    const MyEntity: () = ();
}

inventory::submit!(remotely::__private::NsMember::Interface {
    ns_name: "Watchout",
    name: "MyEntity",
    raw_decl: &<MyEntity as ts_rs::TS>::decl,
    raw_deps: &<MyEntity as ts_rs::TS>::dependencies
});

inventory::submit!(remotely::__private::NsMember::Interface {
    ns_name: "Pixera",
    name: "MyEntity2",
    raw_decl: &<MyEntity2 as ts_rs::TS>::decl,
    raw_deps: &<MyEntity2 as ts_rs::TS>::dependencies
});

impl remotely::__private::Namespace for Watchout {
    const NAME: &'static str = "Watchout";
    type Req = WatchoutReq;
}

inventory::submit!(remotely::__private::NsMember::Method {
    ns_name: "Watchout",
    name: "hello",
    args: &|| vec![
        ("s", <String as ts_rs::TS>::name()),
        ("num", <usize as ts_rs::TS>::name())
    ],

    res: &<usize as ts_rs::TS>::name,
    raw_deps: &<(String, usize, MyEntity) as ts_rs::TS>::dependencies
});

inventory::submit!(remotely::__private::NsMember::Method {
    ns_name: "Watchout",
    name: "hello_stream",
    args: &|| vec![("num", <usize as ts_rs::TS>::name())],
    res: &<() as ts_rs::TS>::name,
    raw_deps: &<(usize,) as ts_rs::TS>::dependencies
});

#[async_trait::async_trait]
impl remotely::__private::Backend for MyBackend {
    fn generate<T>() -> remotely::__private::FileList
    where
        T: remotely::__private::ClientCodegen,
    {
        let mut list = ::std::collections::BTreeMap::new();

        list.insert(::std::path::Path::new("remotely_client.ts"), T::get());

        // repeat for all fields
        list.insert(
            ::std::path::Path::new(concat!("Watchout", ".ts")),
            <Watchout as remotely::__private::Namespace>::code(),
        );

        remotely::__private::FileList::new(list)
    }

    async fn handle_request(&mut self, req: Request, sender: ResponseSender) {
        match req {
            Request::Method(value) => match serde_json::from_value::<MyBackendReq>(value) {
                Ok(evt) => evt.call(self, sender).await,
                Err(err) => {
                    let _ = sender
                        .unbounded_send(
                            serde_json::to_value(&remotely::__private::Error::from(err)).unwrap(),
                        )
                        .ok();
                }
            },
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "namespace")]
enum MyBackendReq {
    Watchout(<Watchout as remotely::__private::Namespace>::Req),
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
    async fn call(self, backend: &mut MyBackend, sender: ResponseSender) {
        match self {
            MyBackendReq::Watchout(method) => method.call(&mut backend.0, sender).await,
        }
    }
}

impl WatchoutReq {
    async fn call(self, ctx: &mut Watchout, sender: ResponseSender) {
        match self {
            WatchoutReq::hello { args } => {
                let res = ctx.hello(args.0, args.1).await;
                let res = serde_json::to_value(res).unwrap();
                sender.unbounded_send(res).unwrap();
            }
            WatchoutReq::hello_stream { args } => {
                let mut s = ctx.hello_stream(args.0);
                tokio::spawn(async move {
                    while let Some(evt) = s.next().await {
                        let res = serde_json::to_value(evt).unwrap();
                        if let Err(err) = sender.unbounded_send(res) {
                            tracing::warn!(?err, "Failed to emit event");
                            break;
                        }
                    }
                });
            }
        }
    }
}
