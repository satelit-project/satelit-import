/// Asks to begin scraping process from specific source
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScrapeIntent {
    /// Intent ID
    #[prost(message, optional, tag = "1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// Indicator from where to scrape data
    #[prost(enumeration = "super::data::Source", tag = "2")]
    pub source: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScrapeIntentResult {
    /// ID of an intent that was used to initiate data scraping
    #[prost(message, optional, tag = "1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// Wherever there's more data to scrape
    #[prost(bool, tag = "2")]
    pub may_continue: bool,
}
/// Represents a task for anime pages scraping
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Task {
    /// Task ID
    #[prost(message, optional, tag = "1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// External DB from where to scrape info
    #[prost(enumeration = "super::data::Source", tag = "2")]
    pub source: i32,
    /// Scraping jobs
    #[prost(message, repeated, tag = "3")]
    pub jobs: ::std::vec::Vec<Job>,
}
/// Represents a single scraping job for an anime page
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Job {
    /// Job ID
    #[prost(message, optional, tag = "1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// Anime ID
    #[prost(sint32, tag = "2")]
    pub anime_id: i32,
}
/// Scrape task creation request
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskCreate {
    /// Maximum number of entities to scrape
    #[prost(sint32, tag = "1")]
    pub limit: i32,
    /// External data source to scrape data from
    #[prost(enumeration = "super::data::Source", tag = "2")]
    pub source: i32,
}
/// Intermediate result of a parse task
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskYield {
    /// ID of the related task
    #[prost(message, optional, tag = "1")]
    pub task_id: ::std::option::Option<super::uuid::Uuid>,
    /// ID of the related job
    #[prost(message, optional, tag = "2")]
    pub job_id: ::std::option::Option<super::uuid::Uuid>,
    /// Parsed anime entity
    #[prost(message, optional, tag = "3")]
    pub anime: ::std::option::Option<super::data::Anime>,
}
/// Signals that a task has been finished
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskFinish {
    /// ID of the related task
    #[prost(message, optional, tag = "1")]
    pub task_id: ::std::option::Option<super::uuid::Uuid>,
}
#[doc = r" Generated client implementations."]
pub mod scraper_service_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = " A service to start scraping process"]
    #[doc = ""]
    #[doc = " 'Scraper' should implement a server side of the service and"]
    #[doc = " something from the outside needs to trigger scraping process."]
    pub struct ScraperServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ScraperServiceClient<tonic::transport::Channel> {
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
    impl<T> ScraperServiceClient<T>
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
        #[doc = " Starts web scraping and returns result of the operation when finished"]
        pub async fn start_scraping(
            &mut self,
            request: impl tonic::IntoRequest<super::ScrapeIntent>,
        ) -> Result<tonic::Response<super::ScrapeIntentResult>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/scraping.ScraperService/StartScraping");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for ScraperServiceClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
}
#[doc = r" Generated client implementations."]
pub mod scraper_tasks_service_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = " A service that manages creation/destruction of scraping tasks"]
    #[doc = ""]
    #[doc = " 'Scraper' will call those methods to initiate scraping and report it's progress"]
    #[doc = " and it's expected to be implemented by 'Importer'."]
    pub struct ScraperTasksServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ScraperTasksServiceClient<tonic::transport::Channel> {
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
    impl<T> ScraperTasksServiceClient<T>
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
        #[doc = " Creates new scraping task and returns info about target to scrape"]
        pub async fn create_task(
            &mut self,
            request: impl tonic::IntoRequest<super::TaskCreate>,
        ) -> Result<tonic::Response<super::Task>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/scraping.ScraperTasksService/CreateTask");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Reports that an atomic piece of data has been scraped"]
        pub async fn yield_result(
            &mut self,
            request: impl tonic::IntoRequest<super::TaskYield>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/scraping.ScraperTasksService/YieldResult");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Reports that scraping has finished and no more work will be done"]
        pub async fn complete_task(
            &mut self,
            request: impl tonic::IntoRequest<super::TaskFinish>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/scraping.ScraperTasksService/CompleteTask");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for ScraperTasksServiceClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod scraper_service_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with ScraperServiceServer."]
    #[async_trait]
    pub trait ScraperService: Send + Sync + 'static {
        #[doc = " Starts web scraping and returns result of the operation when finished"]
        async fn start_scraping(
            &self,
            request: tonic::Request<super::ScrapeIntent>,
        ) -> Result<tonic::Response<super::ScrapeIntentResult>, tonic::Status>;
    }
    #[doc = " A service to start scraping process"]
    #[doc = ""]
    #[doc = " 'Scraper' should implement a server side of the service and"]
    #[doc = " something from the outside needs to trigger scraping process."]
    #[derive(Debug)]
    #[doc(hidden)]
    pub struct ScraperServiceServer<T: ScraperService> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: ScraperService> ScraperServiceServer<T> {
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
    impl<T: ScraperService> Service<http::Request<HyperBody>> for ScraperServiceServer<T> {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<HyperBody>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/scraping.ScraperService/StartScraping" => {
                    struct StartScrapingSvc<T: ScraperService>(pub Arc<T>);
                    impl<T: ScraperService> tonic::server::UnaryService<super::ScrapeIntent> for StartScrapingSvc<T> {
                        type Response = super::ScrapeIntentResult;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ScrapeIntent>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.start_scraping(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = StartScrapingSvc(inner);
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
    impl<T: ScraperService> Clone for ScraperServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: ScraperService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ScraperService> tonic::transport::NamedService for ScraperServiceServer<T> {
        const NAME: &'static str = "scraping.ScraperService";
    }
}
#[doc = r" Generated server implementations."]
pub mod scraper_tasks_service_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with ScraperTasksServiceServer."]
    #[async_trait]
    pub trait ScraperTasksService: Send + Sync + 'static {
        #[doc = " Creates new scraping task and returns info about target to scrape"]
        async fn create_task(
            &self,
            request: tonic::Request<super::TaskCreate>,
        ) -> Result<tonic::Response<super::Task>, tonic::Status>;
        #[doc = " Reports that an atomic piece of data has been scraped"]
        async fn yield_result(
            &self,
            request: tonic::Request<super::TaskYield>,
        ) -> Result<tonic::Response<()>, tonic::Status>;
        #[doc = " Reports that scraping has finished and no more work will be done"]
        async fn complete_task(
            &self,
            request: tonic::Request<super::TaskFinish>,
        ) -> Result<tonic::Response<()>, tonic::Status>;
    }
    #[doc = " A service that manages creation/destruction of scraping tasks"]
    #[doc = ""]
    #[doc = " 'Scraper' will call those methods to initiate scraping and report it's progress"]
    #[doc = " and it's expected to be implemented by 'Importer'."]
    #[derive(Debug)]
    #[doc(hidden)]
    pub struct ScraperTasksServiceServer<T: ScraperTasksService> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: ScraperTasksService> ScraperTasksServiceServer<T> {
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
    impl<T: ScraperTasksService> Service<http::Request<HyperBody>> for ScraperTasksServiceServer<T> {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<HyperBody>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/scraping.ScraperTasksService/CreateTask" => {
                    struct CreateTaskSvc<T: ScraperTasksService>(pub Arc<T>);
                    impl<T: ScraperTasksService> tonic::server::UnaryService<super::TaskCreate> for CreateTaskSvc<T> {
                        type Response = super::Task;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TaskCreate>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.create_task(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = CreateTaskSvc(inner);
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
                "/scraping.ScraperTasksService/YieldResult" => {
                    struct YieldResultSvc<T: ScraperTasksService>(pub Arc<T>);
                    impl<T: ScraperTasksService> tonic::server::UnaryService<super::TaskYield> for YieldResultSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TaskYield>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.yield_result(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = YieldResultSvc(inner);
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
                "/scraping.ScraperTasksService/CompleteTask" => {
                    struct CompleteTaskSvc<T: ScraperTasksService>(pub Arc<T>);
                    impl<T: ScraperTasksService> tonic::server::UnaryService<super::TaskFinish> for CompleteTaskSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TaskFinish>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.complete_task(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = CompleteTaskSvc(inner);
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
    impl<T: ScraperTasksService> Clone for ScraperTasksServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: ScraperTasksService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ScraperTasksService> tonic::transport::NamedService for ScraperTasksServiceServer<T> {
        const NAME: &'static str = "scraping.ScraperTasksService";
    }
}
