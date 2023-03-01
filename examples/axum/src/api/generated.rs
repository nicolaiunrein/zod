use super::{MyBackend, MyEntity, Pixera, Watchout};

const _: () = {
    // ================================================================================================
    // ========================================= Rpc impl =============================================
    // ================================================================================================
    impl ::zod::rpc::__private::codegen::Rpc for Watchout {
        type Req = WatchoutReq;
    }

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
    // ======================================== backend impl ==========================================
    // ================================================================================================
    #[async_trait::async_trait]
    impl ::zod::rpc::__private::server::Backend for MyBackend {
        fn is_member_of_self(member: &'static ::zod::rpc::__private::codegen::RpcMember) -> bool {
            static NAMES: &'static [&'static str] = &[<Watchout as ::zod::Namespace>::NAME];
            NAMES.contains(&member.ns_name())
        }

        async fn handle_request(
            &mut self,
            req: ::zod::rpc::__private::Request,
            sender: ::zod::rpc::__private::ResponseSender,
            subscribers: &mut ::zod::rpc::__private::server::SubscriberMap,
        ) {
            match req {
                ::zod::rpc::__private::Request::Exec { id, value } => {
                    match zod::rpc::__private::serde_json::from_value::<MyBackendReq>(value) {
                        ::std::result::Result::<_, _>::Ok(evt) => {
                            if let Some(jh) = evt.call(id, self, sender).await {
                                subscribers.insert(id, jh);
                            }
                        }
                        ::std::result::Result::<_, _>::Err(err) => {
                            let _ = sender
                                .unbounded_send(::zod::rpc::__private::Response::error(id, err))
                                .ok();
                        }
                    }
                }
                ::zod::rpc::__private::Request::CancelStream { id } => {
                    if let Some(jh) = subscribers.remove(&id) {
                        jh.abort();
                    }
                }
            }
        }
    }

    // ================================================================================================
    // ====================================== BackendReq impl =========================================
    // ================================================================================================
    #[derive(::zod::rpc::__private::serde::Deserialize, Debug)]
    #[serde(tag = "namespace")]
    enum MyBackendReq {
        Watchout(<Watchout as ::zod::rpc::__private::codegen::Rpc>::Req),
        Pixera(<Pixera as ::zod::rpc::__private::codegen::Rpc>::Req),
    }

    impl MyBackendReq {
        async fn call(
            self,
            id: usize,
            backend: &mut MyBackend,
            sender: ::zod::rpc::__private::ResponseSender,
        ) -> ::std::option::Option<::zod::rpc::__private::tokio::task::JoinHandle<()>> {
            match self {
                MyBackendReq::Watchout(req) => req.call(id, &mut backend.0, sender).await,
                MyBackendReq::Pixera(req) => req.call(id, &mut backend.1, sender).await,
            }
        }
    }

    // ================================================================================================
    // ===================================== NamespaceReq impl ========================================
    // ================================================================================================
    #[derive(::zod::rpc::__private::serde::Deserialize, Debug)]
    #[serde(tag = "method")]
    #[allow(non_camel_case_types)]
    #[allow(non_snake_case)]
    #[allow(non_upper_case_globals)]
    pub enum WatchoutReq {
        hello { args: (String, usize) },
        hello_stream { args: (usize,) },
    }

    impl WatchoutReq {
        async fn call(
            self,
            id: usize,
            ctx: &mut Watchout,
            sender: ::zod::rpc::__private::ResponseSender,
        ) -> ::std::option::Option<::zod::rpc::__private::tokio::task::JoinHandle<()>> {
            match self {
                WatchoutReq::hello { args } => {
                    let res = ctx.hello(args.0, args.1).await;
                    sender
                        .unbounded_send(::zod::rpc::__private::Response::method(id, res))
                        .unwrap();
                    None
                }
                WatchoutReq::hello_stream { args } => {
                    let s = ctx.hello_stream(args.0);
                    Some(::zod::rpc::__private::tokio::spawn(async move {
                        ::zod::rpc::__private::futures::pin_mut!(s);
                        while let ::std::option::Option::Some(evt) =
                            ::zod::rpc::__private::futures::StreamExt::next(&mut s).await
                        {
                            if let ::std::result::Result::<_, _>::Err(err) = sender
                                .unbounded_send(::zod::rpc::__private::Response::stream(id, evt))
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
        async fn call(
            self,
            _id: usize,
            _ctx: &mut Pixera,
            _sender: ::zod::rpc::__private::ResponseSender,
        ) -> ::std::option::Option<::zod::rpc::__private::tokio::task::JoinHandle<()>> {
            None
        }
    }
};
