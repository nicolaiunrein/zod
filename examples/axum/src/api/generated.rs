use super::{MyEntity, Pixera, Watchout};

const _: () = {
    // ================================================================================================
    // ========================================= Rpc impl =============================================
    // ================================================================================================
    // impl ::zod::rpc::__private::codegen::Rpc for Watchout {
    // type Req = WatchoutReq;
    // }

    // ================================================================================================
    // ======================================== method impl ===========================================
    // ================================================================================================
    ::zod::rpc::__private::inventory::submit!(::zod::rpc::__private::codegen::RpcMember::Method {
        ns_name: <Watchout as ::zod::Namespace>::NAME,
        name: "hello",
        args: &|| vec![
            ::zod::rpc::__private::codegen::RpcArgument::new::<String>("s"),
            ::zod::rpc::__private::codegen::RpcArgument::new::<usize>("num"),
        ],
        res: &<usize as ::zod::ZodType>::type_def,
    });

    // ================================================================================================
    // ======================================== stream impl ===========================================
    // ================================================================================================
    ::zod::rpc::__private::inventory::submit!(::zod::rpc::__private::codegen::RpcMember::Stream {
        ns_name: <Watchout as ::zod::Namespace>::NAME,
        name: "hello_stream",
        args: &|| vec![::zod::rpc::__private::codegen::RpcArgument::new::<usize>(
            "num"
        ),],
        res: &<usize as ::zod::ZodType>::type_def,
    });

    // ================================================================================================
    // ===================================== NamespaceReq impl ========================================
    // ================================================================================================
    // #[derive(::zod::rpc::__private::serde::Deserialize, Debug)]
    // #[serde(tag = "method")]
    // #[allow(non_camel_case_types)]
    // #[allow(non_snake_case)]
    // #[allow(non_upper_case_globals)]
    // pub enum WatchoutReq {
    // hello { args: (String, usize) },
    // hello_stream { args: (usize,) },
    // }

    // impl WatchoutReq {
    // pub async fn call(
    // self,
    // id: usize,
    // ctx: &mut Watchout,
    // sender: ::zod::rpc::__private::ResponseSender,
    // ) -> ::std::option::Option<::zod::rpc::__private::tokio::task::JoinHandle<()>> {
    // match self {
    // WatchoutReq::hello { args } => {
    // let res = ctx.hello(args.0, args.1).await;
    // sender
    // .unbounded_send(::zod::rpc::__private::Response::method(id, res))
    // .unwrap();
    // None
    // }
    // WatchoutReq::hello_stream { args } => {
    // let s = ctx.hello_stream(args.0);
    // Some(::zod::rpc::__private::tokio::spawn(async move {
    // ::zod::rpc::__private::futures::pin_mut!(s);
    // while let ::std::option::Option::Some(evt) =
    // ::zod::rpc::__private::futures::StreamExt::next(&mut s).await
    // {
    // if let ::std::result::Result::<_, _>::Err(err) = sender
    // .unbounded_send(::zod::rpc::__private::Response::stream(id, evt))
    // {
    // tracing::warn!(?err, "Failed to emit event");
    // break;
    // }
    // }
    // }))
    // }
    // }
    // }
    // }

    // ================================================================================================
    // ===================================== ** Duplicates  ** ========================================
    // ================================================================================================

    impl ::zod::rpc::__private::codegen::Rpc for Pixera {
        type Req = PixeraReq;
    }

    ::zod::rpc::__private::inventory::submit!(::zod::rpc::__private::codegen::RpcMember::Method {
        ns_name: <Watchout as ::zod::Namespace>::NAME,
        name: "nested",
        args: &|| vec![::zod::rpc::__private::codegen::RpcArgument::new::<MyEntity>("value"),],
        res: &<usize as zod::ZodType>::type_def,
    });

    #[derive(::zod::rpc::__private::serde::Deserialize, Debug)]
    #[serde(tag = "method")]
    #[allow(non_camel_case_types)]
    #[allow(non_snake_case)]
    #[allow(non_upper_case_globals)]
    pub enum PixeraReq {}

    impl PixeraReq {
        pub async fn call(
            self,
            _id: usize,
            _ctx: &mut Pixera,
            _sender: ::zod::rpc::__private::ResponseSender,
        ) -> ::std::option::Option<::zod::rpc::__private::tokio::task::JoinHandle<()>> {
            None
        }
    }
};
