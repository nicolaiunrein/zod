use super::{MyBackend, MyEntity, Pixera, Watchout};

const _: () = {
    // Prevent duplicate interfaces
    impl WatchoutReq {
        #[allow(dead_code)]
        #[allow(non_upper_case_globals)]
        const MyEntity: () = ();
    }

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
        ns_name: "Watchout",
        name: "hello",
        args: &|| vec![
            (
                "s",
                <String as ::zod::Codegen>::type_def(),
                <String as ::zod::Codegen>::schema()
            ),
            (
                "num",
                <usize as ::zod::Codegen>::type_def(),
                <usize as ::zod::Codegen>::schema()
            )
        ],

        res: &<usize as ::zod::Codegen>::type_def,
    });

    // ================================================================================================
    // ======================================== stream impl ===========================================
    // ================================================================================================
    ::zod::rpc::__private::inventory::submit!(::zod::rpc::__private::codegen::RpcMember::Stream {
        ns_name: "Watchout",
        name: "hello_stream",
        args: &|| vec![(
            "num",
            <usize as ::zod::Codegen>::type_def(),
            <usize as ::zod::Codegen>::schema()
        )],
        res: &<usize as ::zod::Codegen>::type_def,
    });

    // ================================================================================================
    // ======================================== backend impl ==========================================
    // ================================================================================================
    #[async_trait::async_trait]
    impl ::zod::rpc::__private::server::Backend for MyBackend {
        fn generate<T>() -> String
        where
            T: ::zod::rpc::__private::codegen::ClientCodegen,
        {
            let mut code = T::get();
            // repeat for all namespaces
            code.push_str(&<Watchout as ::zod::rpc::__private::codegen::Rpc>::code());
            code.push_str(&<Pixera as ::zod::rpc::__private::codegen::Rpc>::code());
            code
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
                        Ok(evt) => {
                            if let Some(jh) = evt.call(id, self, sender).await {
                                subscribers.insert(id, jh);
                            }
                        }
                        Err(err) => {
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
    #[derive(
        ::zod::rpc::__private::serde::Serialize, ::zod::rpc::__private::serde::Deserialize, Debug,
    )]
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
        ) -> Option<::zod::rpc::__private::tokio::task::JoinHandle<()>> {
            match self {
                MyBackendReq::Watchout(req) => req.call(id, &mut backend.0, sender).await,
                MyBackendReq::Pixera(req) => req.call(id, &mut backend.1, sender).await,
            }
        }
    }

    // ================================================================================================
    // ===================================== NamespaceReq impl ========================================
    // ================================================================================================
    #[derive(
        ::zod::rpc::__private::serde::Serialize, ::zod::rpc::__private::serde::Deserialize, Debug,
    )]
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
        ) -> Option<::zod::rpc::__private::tokio::task::JoinHandle<()>> {
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
                        futures::pin_mut!(s);
                        while let Some(evt) =
                            ::zod::rpc::__private::futures::StreamExt::next(&mut s).await
                        {
                            if let Err(err) = sender
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
    impl Pixera {
        #[allow(dead_code)]
        #[allow(non_upper_case_globals)]
        const MyEntity2: () = ();
    }

    impl ::zod::rpc::__private::codegen::Rpc for Pixera {
        type Req = PixeraReq;
    }

    ::zod::rpc::__private::inventory::submit!(::zod::rpc::__private::codegen::RpcMember::Method {
        ns_name: "Watchout",
        name: "nested",
        args: &|| vec![(
            "value",
            <MyEntity as zod::Codegen>::type_def(),
            <MyEntity as zod::Codegen>::schema()
        )],

        res: &<usize as zod::Codegen>::type_def,
    });

    #[derive(
        ::zod::rpc::__private::serde::Serialize, ::zod::rpc::__private::serde::Deserialize, Debug,
    )]
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
        ) -> Option<::zod::rpc::__private::tokio::task::JoinHandle<()>> {
            None
        }
    }
};
