/// Asks to import anime titles index and schedule new titles for scraping
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportIntent {
    /// Intent ID
    #[prost(message, optional, tag = "1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// External data source to which index files belongs to
    #[prost(enumeration = "super::data::Source", tag = "2")]
    pub source: i32,
    /// URL of latest anime titles index
    #[prost(string, tag = "3")]
    pub new_index_url: std::string::String,
    /// URL of previous anime titles index
    #[prost(string, tag = "4")]
    pub old_index_url: std::string::String,
    /// Identifiers of anime titles that should be re-imported
    #[prost(sint32, repeated, tag = "5")]
    pub reimport_ids: ::std::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportIntentResult {
    /// Intent ID
    #[prost(message, optional, tag = "1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// IDs of anime titles that was not imported
    #[prost(sint32, repeated, tag = "2")]
    pub skipped_ids: ::std::vec::Vec<i32>,
}
#[doc = r" Generated client implementations."]
pub mod import_service_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = " A service to start raw data import"]
    #[doc = ""]
    #[doc = " 'Importer' should implement the service and start importing a raw data when requested"]
    #[doc = " such as AniDB database dump that will be used to produce scraping tasks."]
    pub struct ImportServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ImportServiceClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ImportServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        #[doc = " Start import process of raw data and returns result of the operation when finished"]
        pub async fn start_import(
            &mut self,
            request: impl tonic::IntoRequest<super::ImportIntent>,
        ) -> Result<tonic::Response<super::ImportIntentResult>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/import.ImportService/StartImport");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for ImportServiceClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod import_service_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with ImportServiceServer."]
    #[async_trait]
    pub trait ImportService: Send + Sync + 'static {
        #[doc = " Start import process of raw data and returns result of the operation when finished"]
        async fn start_import(
            &self,
            request: tonic::Request<super::ImportIntent>,
        ) -> Result<tonic::Response<super::ImportIntentResult>, tonic::Status>;
    }
    #[doc = " A service to start raw data import"]
    #[doc = ""]
    #[doc = " 'Importer' should implement the service and start importing a raw data when requested"]
    #[doc = " such as AniDB database dump that will be used to produce scraping tasks."]
    #[derive(Debug)]
    #[doc(hidden)]
    pub struct ImportServiceServer<T: ImportService> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: ImportService> ImportServiceServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, None);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, Some(interceptor.into()));
            Self { inner }
        }
    }
    impl<T: ImportService> Service<http::Request<HyperBody>> for ImportServiceServer<T> {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<HyperBody>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/import.ImportService/StartImport" => {
                    struct StartImportSvc<T: ImportService>(pub Arc<T>);
                    impl<T: ImportService> tonic::server::UnaryService<super::ImportIntent> for StartImportSvc<T> {
                        type Response = super::ImportIntentResult;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ImportIntent>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.start_import(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = StartImportSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .body(tonic::body::BoxBody::empty())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: ImportService> Clone for ImportServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: ImportService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ImportService> tonic::transport::NamedService for ImportServiceServer<T> {
        const NAME: &'static str = "import.ImportService";
    }
}
