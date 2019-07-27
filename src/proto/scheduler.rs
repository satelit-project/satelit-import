/// Asks to import anime titles index and schedule new titles for scraping
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportIntent {
    /// Intent ID
    #[prost(string, tag="1")]
    pub id: std::string::String,
    /// Represents an external DB from where anime titles index should be imported
    #[prost(enumeration="super::data::Source", tag="2")]
    pub source: i32,
    /// URL of anime titles dump location
    #[prost(string, tag="3")]
    pub dump_url: std::string::String,
    /// Identifiers of anime titles that should be re-imported
    #[prost(sint32, repeated, tag="4")]
    pub reimport_ids: ::std::vec::Vec<i32>,
    /// URL to send request with import result
    #[prost(string, tag="5")]
    pub callback_url: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportIntentResult {
    /// Intent ID
    #[prost(string, tag="1")]
    pub id: std::string::String,
    /// If import succeeded then `true`, `false` otherwise
    #[prost(bool, tag="2")]
    pub succeeded: bool,
    /// IDs of anime titles that was not imported
    #[prost(sint32, repeated, tag="3")]
    pub skipped_ids: ::std::vec::Vec<i32>,
    /// Description of the error if import failed
    #[prost(string, tag="4")]
    pub error_description: std::string::String,
}
pub mod client {
    use ::tower_grpc::codegen::client::*;
    use super::{ImportIntent, ImportIntentResult};

    /// A service to start raw data import
    /// 
    /// 'Importer' should implement the service and start importing a raw data when requested
    /// such as AniDB database dump that will be used to produce scraping tasks.
    #[derive(Debug, Clone)]
    pub struct ImportService<T> {
        inner: grpc::Grpc<T>,
    }

    impl<T> ImportService<T> {
        pub fn new(inner: T) -> Self {
            let inner = grpc::Grpc::new(inner);
            Self { inner }
        }

        /// Poll whether this client is ready to send another request.
        pub fn poll_ready<R>(&mut self) -> futures::Poll<(), grpc::Status>
        where T: grpc::GrpcService<R>,
        {
            self.inner.poll_ready()
        }

        /// Get a `Future` of when this client is ready to send another request.
        pub fn ready<R>(self) -> impl futures::Future<Item = Self, Error = grpc::Status>
        where T: grpc::GrpcService<R>,
        {
            futures::Future::map(self.inner.ready(), |inner| Self { inner })
        }

        /// A service to start raw data import
        /// 
        /// 'Importer' should implement the service and start importing a raw data when requested
        /// such as AniDB database dump that will be used to produce scraping tasks.
        pub fn start_import<R>(&mut self, request: grpc::Request<ImportIntent>) -> grpc::unary::ResponseFuture<(), T::Future, T::ResponseBody>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<ImportIntent>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/scheduler.ImportService/StartImport");
            self.inner.unary(request, path)
        }
    }

    /// A service that receives callbacks from `ImportService` about import progress
    /// 
    /// 'Importer' will call those methods to report import progress and it's expected to
    /// be implemented by 'Scheduler'.
    #[derive(Debug, Clone)]
    pub struct ImportHooksService<T> {
        inner: grpc::Grpc<T>,
    }

    impl<T> ImportHooksService<T> {
        pub fn new(inner: T) -> Self {
            let inner = grpc::Grpc::new(inner);
            Self { inner }
        }

        /// Poll whether this client is ready to send another request.
        pub fn poll_ready<R>(&mut self) -> futures::Poll<(), grpc::Status>
        where T: grpc::GrpcService<R>,
        {
            self.inner.poll_ready()
        }

        /// Get a `Future` of when this client is ready to send another request.
        pub fn ready<R>(self) -> impl futures::Future<Item = Self, Error = grpc::Status>
        where T: grpc::GrpcService<R>,
        {
            futures::Future::map(self.inner.ready(), |inner| Self { inner })
        }

        /// A service that receives callbacks from `ImportService` about import progress
        /// 
        /// 'Importer' will call those methods to report import progress and it's expected to
        /// be implemented by 'Scheduler'.
        pub fn import_finished<R>(&mut self, request: grpc::Request<ImportIntentResult>) -> grpc::unary::ResponseFuture<(), T::Future, T::ResponseBody>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<ImportIntentResult>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/scheduler.ImportHooksService/ImportFinished");
            self.inner.unary(request, path)
        }
    }
}

pub mod server {
    use ::tower_grpc::codegen::server::*;
    use super::{ImportIntent, ImportIntentResult};

    // Redefine the try_ready macro so that it doesn't need to be explicitly
    // imported by the user of this generated code.
    macro_rules! try_ready {
        ($e:expr) => (match $e {
            Ok(futures::Async::Ready(t)) => t,
            Ok(futures::Async::NotReady) => return Ok(futures::Async::NotReady),
            Err(e) => return Err(From::from(e)),
        })
    }

    /// A service to start raw data import
    /// 
    /// 'Importer' should implement the service and start importing a raw data when requested
    /// such as AniDB database dump that will be used to produce scraping tasks.
    pub trait ImportService: Clone {
        type StartImportFuture: futures::Future<Item = grpc::Response<()>, Error = grpc::Status>;

        /// Begins import process of raw data
        fn start_import(&mut self, request: grpc::Request<ImportIntent>) -> Self::StartImportFuture;
    }

    #[derive(Debug, Clone)]
    pub struct ImportServiceServer<T> {
        import_service: T,
    }

    impl<T> ImportServiceServer<T>
    where T: ImportService,
    {
        pub fn new(import_service: T) -> Self {
            Self { import_service }
        }
    }

    impl<T> tower::Service<http::Request<grpc::BoxBody>> for ImportServiceServer<T>
    where T: ImportService,
    {
        type Response = http::Response<import_service::ResponseBody<T>>;
        type Error = grpc::Never;
        type Future = import_service::ResponseFuture<T>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(().into())
        }

        fn call(&mut self, request: http::Request<grpc::BoxBody>) -> Self::Future {
            use self::import_service::Kind::*;

            match request.uri().path() {
                "/scheduler.ImportService/StartImport" => {
                    let service = import_service::methods::StartImport(self.import_service.clone());
                    let response = grpc::unary(service, request);
                    import_service::ResponseFuture { kind: StartImport(response) }
                }
                _ => {
                    import_service::ResponseFuture { kind: __Generated__Unimplemented(grpc::unimplemented(format!("unknown service: {:?}", request.uri().path()))) }
                }
            }
        }
    }

    impl<T> tower::Service<()> for ImportServiceServer<T>
    where T: ImportService,
    {
        type Response = Self;
        type Error = grpc::Never;
        type Future = futures::FutureResult<Self::Response, Self::Error>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(futures::Async::Ready(()))
        }

        fn call(&mut self, _target: ()) -> Self::Future {
            futures::ok(self.clone())
        }
    }

    impl<T> tower::Service<http::Request<tower_hyper::Body>> for ImportServiceServer<T>
    where T: ImportService,
    {
        type Response = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Response;
        type Error = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Error;
        type Future = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Future;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            tower::Service::<http::Request<grpc::BoxBody>>::poll_ready(self)
        }

        fn call(&mut self, request: http::Request<tower_hyper::Body>) -> Self::Future {
            let request = request.map(|b| grpc::BoxBody::map_from(b));
            tower::Service::<http::Request<grpc::BoxBody>>::call(self, request)
        }
    }

    pub mod import_service {
        use ::tower_grpc::codegen::server::*;
        use super::ImportService;
        use super::super::ImportIntent;

        pub struct ResponseFuture<T>
        where T: ImportService,
        {
            pub(super) kind: Kind<
                // StartImport
                grpc::unary::ResponseFuture<methods::StartImport<T>, grpc::BoxBody, ImportIntent>,
                // A generated catch-all for unimplemented service calls
                grpc::unimplemented::ResponseFuture,
            >,
        }

        impl<T> futures::Future for ResponseFuture<T>
        where T: ImportService,
        {
            type Item = http::Response<ResponseBody<T>>;
            type Error = grpc::Never;

            fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    StartImport(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: StartImport(body) }
                        });
                        Ok(response.into())
                    }
                    __Generated__Unimplemented(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: __Generated__Unimplemented(body) }
                        });
                        Ok(response.into())
                    }
                }
            }
        }

        pub struct ResponseBody<T>
        where T: ImportService,
        {
            pub(super) kind: Kind<
                // StartImport
                grpc::Encode<grpc::unary::Once<<methods::StartImport<T> as grpc::UnaryService<ImportIntent>>::Response>>,
                // A generated catch-all for unimplemented service calls
                (),
            >,
        }

        impl<T> tower::HttpBody for ResponseBody<T>
        where T: ImportService,
        {
            type Data = <grpc::BoxBody as grpc::Body>::Data;
            type Error = grpc::Status;

            fn is_end_stream(&self) -> bool {
                use self::Kind::*;

                match self.kind {
                    StartImport(ref v) => v.is_end_stream(),
                    __Generated__Unimplemented(_) => true,
                }
            }

            fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    StartImport(ref mut v) => v.poll_data(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }

            fn poll_trailers(&mut self) -> futures::Poll<Option<http::HeaderMap>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    StartImport(ref mut v) => v.poll_trailers(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }
        }

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub(super) enum Kind<StartImport, __Generated__Unimplemented> {
            StartImport(StartImport),
            __Generated__Unimplemented(__Generated__Unimplemented),
        }

        pub mod methods {
            use ::tower_grpc::codegen::server::*;
            use super::super::{ImportService, ImportIntent};

            pub struct StartImport<T>(pub T);

            impl<T> tower::Service<grpc::Request<ImportIntent>> for StartImport<T>
            where T: ImportService,
            {
                type Response = grpc::Response<()>;
                type Error = grpc::Status;
                type Future = T::StartImportFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<ImportIntent>) -> Self::Future {
                    self.0.start_import(request)
                }
            }
        }
    }

    // Redefine the try_ready macro so that it doesn't need to be explicitly
    // imported by the user of this generated code.
    macro_rules! try_ready {
        ($e:expr) => (match $e {
            Ok(futures::Async::Ready(t)) => t,
            Ok(futures::Async::NotReady) => return Ok(futures::Async::NotReady),
            Err(e) => return Err(From::from(e)),
        })
    }

    /// A service that receives callbacks from `ImportService` about import progress
    /// 
    /// 'Importer' will call those methods to report import progress and it's expected to
    /// be implemented by 'Scheduler'.
    pub trait ImportHooksService: Clone {
        type ImportFinishedFuture: futures::Future<Item = grpc::Response<()>, Error = grpc::Status>;

        /// Reports that import has finished
        fn import_finished(&mut self, request: grpc::Request<ImportIntentResult>) -> Self::ImportFinishedFuture;
    }

    #[derive(Debug, Clone)]
    pub struct ImportHooksServiceServer<T> {
        import_hooks_service: T,
    }

    impl<T> ImportHooksServiceServer<T>
    where T: ImportHooksService,
    {
        pub fn new(import_hooks_service: T) -> Self {
            Self { import_hooks_service }
        }
    }

    impl<T> tower::Service<http::Request<grpc::BoxBody>> for ImportHooksServiceServer<T>
    where T: ImportHooksService,
    {
        type Response = http::Response<import_hooks_service::ResponseBody<T>>;
        type Error = grpc::Never;
        type Future = import_hooks_service::ResponseFuture<T>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(().into())
        }

        fn call(&mut self, request: http::Request<grpc::BoxBody>) -> Self::Future {
            use self::import_hooks_service::Kind::*;

            match request.uri().path() {
                "/scheduler.ImportHooksService/ImportFinished" => {
                    let service = import_hooks_service::methods::ImportFinished(self.import_hooks_service.clone());
                    let response = grpc::unary(service, request);
                    import_hooks_service::ResponseFuture { kind: ImportFinished(response) }
                }
                _ => {
                    import_hooks_service::ResponseFuture { kind: __Generated__Unimplemented(grpc::unimplemented(format!("unknown service: {:?}", request.uri().path()))) }
                }
            }
        }
    }

    impl<T> tower::Service<()> for ImportHooksServiceServer<T>
    where T: ImportHooksService,
    {
        type Response = Self;
        type Error = grpc::Never;
        type Future = futures::FutureResult<Self::Response, Self::Error>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(futures::Async::Ready(()))
        }

        fn call(&mut self, _target: ()) -> Self::Future {
            futures::ok(self.clone())
        }
    }

    impl<T> tower::Service<http::Request<tower_hyper::Body>> for ImportHooksServiceServer<T>
    where T: ImportHooksService,
    {
        type Response = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Response;
        type Error = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Error;
        type Future = <Self as tower::Service<http::Request<grpc::BoxBody>>>::Future;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            tower::Service::<http::Request<grpc::BoxBody>>::poll_ready(self)
        }

        fn call(&mut self, request: http::Request<tower_hyper::Body>) -> Self::Future {
            let request = request.map(|b| grpc::BoxBody::map_from(b));
            tower::Service::<http::Request<grpc::BoxBody>>::call(self, request)
        }
    }

    pub mod import_hooks_service {
        use ::tower_grpc::codegen::server::*;
        use super::ImportHooksService;
        use super::super::ImportIntentResult;

        pub struct ResponseFuture<T>
        where T: ImportHooksService,
        {
            pub(super) kind: Kind<
                // ImportFinished
                grpc::unary::ResponseFuture<methods::ImportFinished<T>, grpc::BoxBody, ImportIntentResult>,
                // A generated catch-all for unimplemented service calls
                grpc::unimplemented::ResponseFuture,
            >,
        }

        impl<T> futures::Future for ResponseFuture<T>
        where T: ImportHooksService,
        {
            type Item = http::Response<ResponseBody<T>>;
            type Error = grpc::Never;

            fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    ImportFinished(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: ImportFinished(body) }
                        });
                        Ok(response.into())
                    }
                    __Generated__Unimplemented(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: __Generated__Unimplemented(body) }
                        });
                        Ok(response.into())
                    }
                }
            }
        }

        pub struct ResponseBody<T>
        where T: ImportHooksService,
        {
            pub(super) kind: Kind<
                // ImportFinished
                grpc::Encode<grpc::unary::Once<<methods::ImportFinished<T> as grpc::UnaryService<ImportIntentResult>>::Response>>,
                // A generated catch-all for unimplemented service calls
                (),
            >,
        }

        impl<T> tower::HttpBody for ResponseBody<T>
        where T: ImportHooksService,
        {
            type Data = <grpc::BoxBody as grpc::Body>::Data;
            type Error = grpc::Status;

            fn is_end_stream(&self) -> bool {
                use self::Kind::*;

                match self.kind {
                    ImportFinished(ref v) => v.is_end_stream(),
                    __Generated__Unimplemented(_) => true,
                }
            }

            fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    ImportFinished(ref mut v) => v.poll_data(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }

            fn poll_trailers(&mut self) -> futures::Poll<Option<http::HeaderMap>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    ImportFinished(ref mut v) => v.poll_trailers(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }
        }

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub(super) enum Kind<ImportFinished, __Generated__Unimplemented> {
            ImportFinished(ImportFinished),
            __Generated__Unimplemented(__Generated__Unimplemented),
        }

        pub mod methods {
            use ::tower_grpc::codegen::server::*;
            use super::super::{ImportHooksService, ImportIntentResult};

            pub struct ImportFinished<T>(pub T);

            impl<T> tower::Service<grpc::Request<ImportIntentResult>> for ImportFinished<T>
            where T: ImportHooksService,
            {
                type Response = grpc::Response<()>;
                type Error = grpc::Status;
                type Future = T::ImportFinishedFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<ImportIntentResult>) -> Self::Future {
                    self.0.import_finished(request)
                }
            }
        }
    }
}
