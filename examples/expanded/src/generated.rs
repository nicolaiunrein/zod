use crate::remotely_core::{ClientCodegen, Namespace};

use super::*;

impl Watchout {
    #[allow(dead_code)]
    #[allow(non_upper_case_globals)]
    const MyEntity: () = ();
}

inventory::submit!(crate::remotely_core::NsMember::Interface {
    ns_name: "Watchout",
    name: "MyEntity",
    raw_decl: &|| <MyEntity as ts_rs::TS>::decl(),
    raw_deps: &|| <MyEntity as ts_rs::TS>::dependencies()
});

inventory::submit!(crate::remotely_core::NsMember::Interface {
    ns_name: "Pixera",
    name: "MyEntity2",
    raw_decl: &|| <MyEntity2 as ts_rs::TS>::decl(),
    raw_deps: &|| <MyEntity2 as ts_rs::TS>::dependencies()
});

impl remotely_core::Namespace for Watchout {
    const NAME: &'static str = "Watchout";
    type Req = WatchoutReq;
}

inventory::submit!(crate::remotely_core::NsMember::Method {
    ns_name: "Watchout",
    name: "hello",
    args: &|| vec![
        ("s", <String as ts_rs::TS>::name()),
        ("num", <usize as ts_rs::TS>::name())
    ],

    res: &|| <usize as ts_rs::TS>::name(),
    raw_deps: &|| <(String, usize, MyEntity) as ts_rs::TS>::dependencies()
});

#[async_trait::async_trait]
impl remotely_core::Backend for MyBackend {
    fn generate<T>() -> remotely_core::FileList
    where
        T: ClientCodegen,
    {
        let mut list = ::std::collections::BTreeMap::new();

        list.insert(::std::path::Path::new("remotely_client.ts"), T::get());

        // repeat for all fields
        list.insert(
            ::std::path::Path::new(concat!("Watchout", ".ts")),
            <Watchout as remotely_core::Namespace>::code(),
        );

        remotely_core::FileList::new(list)
    }

    async fn handle_request(&mut self, req: serde_json::Value) -> serde_json::Value {
        let evt: MyBackendReq = serde_json::from_value(req).unwrap();
        let res = evt.call(self).await;
        res
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "namespace")]
enum MyBackendReq {
    Watchout(<Watchout as Namespace>::Req),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "method")]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub enum WatchoutReq {
    hello { args: (String, usize) },
}

impl MyBackendReq {
    async fn call(self, backend: &mut MyBackend) -> serde_json::Value {
        match self {
            MyBackendReq::Watchout(method) => method.call(&mut backend.0).await,
        }
    }
}

impl WatchoutReq {
    async fn call(self, ctx: &mut Watchout) -> serde_json::Value {
        match self {
            WatchoutReq::hello { args } => {
                let res = ctx.hello(args.0, args.1).await;
                serde_json::to_value(res).unwrap()
            }
        }
    }
}
