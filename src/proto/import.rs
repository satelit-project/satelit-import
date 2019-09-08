/// Asks to import anime titles index and schedule new titles for scraping
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportIntent {
    /// Intent ID
    #[prost(message, optional, tag="1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// External data source to which index files belongs to
    #[prost(enumeration="super::data::Source", tag="2")]
    pub source: i32,
    /// URL of latest anime titles index
    #[prost(string, tag="3")]
    pub new_index_url: std::string::String,
    /// URL of previous anime titles index
    #[prost(string, tag="4")]
    pub old_index_url: std::string::String,
    /// Identifiers of anime titles that should be re-imported
    #[prost(sint32, repeated, tag="5")]
    pub reimport_ids: ::std::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportIntentResult {
    /// Intent ID
    #[prost(message, optional, tag="1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// IDs of anime titles that was not imported
    #[prost(sint32, repeated, tag="2")]
    pub skipped_ids: ::std::vec::Vec<i32>,
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
        pub fn start_import<R>(&mut self, request: grpc::Request<ImportIntent>) -> grpc::unary::ResponseFuture<ImportIntentResult, T::Future, T::ResponseBody>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<ImportIntent>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/import.ImportService/StartImport");
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
        type StartImportFuture: futures::Future<Item = grpc::Response<ImportIntentResult>, Error = grpc::Status>;

        /// Start import process of raw data and returns result of the operation when finished
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
                "/import.ImportService/StartImport" => {
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
            use super::super::{ImportService, ImportIntent, ImportIntentResult};

            pub struct StartImport<T>(pub T);

            impl<T> tower::Service<grpc::Request<ImportIntent>> for StartImport<T>
            where T: ImportService,
            {
                type Response = grpc::Response<ImportIntentResult>;
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
}
