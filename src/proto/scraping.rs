/// Asks to begin scraping process from specific source
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScrapeIntent {
    /// Intent ID
    #[prost(message, optional, tag="1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// Indicator from where to scrape data
    #[prost(enumeration="super::data::Source", tag="2")]
    pub source: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScrapeIntentResult {
    /// ID of an intent that was used to initiate data scraping
    #[prost(message, optional, tag="1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// Wherever there's more data to scrape
    #[prost(bool, tag="2")]
    pub may_continue: bool,
}
/// Represents a task for anime pages scraping
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Task {
    /// Task ID
    #[prost(message, optional, tag="1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// External DB from where to scrape info
    #[prost(enumeration="super::data::Source", tag="2")]
    pub source: i32,
    /// Scraping jobs
    #[prost(message, repeated, tag="3")]
    pub jobs: ::std::vec::Vec<Job>,
}
/// Represents a single scraping job for an anime page
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Job {
    /// Job ID
    #[prost(message, optional, tag="1")]
    pub id: ::std::option::Option<super::uuid::Uuid>,
    /// Anime ID
    #[prost(sint32, tag="2")]
    pub anime_id: i32,
}
/// Scrape task creation request
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskCreate {
    /// Maximum number of entities to scrape
    #[prost(sint32, tag="1")]
    pub limit: i32,
    /// External data source to scrape data from
    #[prost(enumeration="super::data::Source", tag="2")]
    pub source: i32,
}
/// Intermediate result of a parse task
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskYield {
    /// ID of the related task
    #[prost(message, optional, tag="1")]
    pub task_id: ::std::option::Option<super::uuid::Uuid>,
    /// ID of the related job
    #[prost(message, optional, tag="2")]
    pub job_id: ::std::option::Option<super::uuid::Uuid>,
    /// Parsed anime entity
    #[prost(message, optional, tag="3")]
    pub anime: ::std::option::Option<super::data::Anime>,
}
/// Signals that a task has been finished
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskFinish {
    /// ID of the related task
    #[prost(message, optional, tag="1")]
    pub task_id: ::std::option::Option<super::uuid::Uuid>,
}
pub mod client {
    use ::tower_grpc::codegen::client::*;
    use super::{ScrapeIntent, ScrapeIntentResult, TaskCreate, Task, TaskYield, TaskFinish};

    /// A service to start scraping process
    /// 
    /// 'Scraper' should implement a server side of the service and
    /// something from the outside needs to trigger scraping process.
    #[derive(Debug, Clone)]
    pub struct ScraperService<T> {
        inner: grpc::Grpc<T>,
    }

    impl<T> ScraperService<T> {
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

        /// A service to start scraping process
        /// 
        /// 'Scraper' should implement a server side of the service and
        /// something from the outside needs to trigger scraping process.
        pub fn start_scraping<R>(&mut self, request: grpc::Request<ScrapeIntent>) -> grpc::unary::ResponseFuture<ScrapeIntentResult, T::Future, T::ResponseBody>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<ScrapeIntent>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/scraping.ScraperService/StartScraping");
            self.inner.unary(request, path)
        }
    }

    /// A service that manages creation/destruction of scraping tasks
    /// 
    /// 'Scraper' will call those methods to initiate scraping and report it's progress
    /// and it's expected to be implemented by 'Importer'.
    #[derive(Debug, Clone)]
    pub struct ScraperTasksService<T> {
        inner: grpc::Grpc<T>,
    }

    impl<T> ScraperTasksService<T> {
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

        /// A service that manages creation/destruction of scraping tasks
        /// 
        /// 'Scraper' will call those methods to initiate scraping and report it's progress
        /// and it's expected to be implemented by 'Importer'.
        pub fn create_task<R>(&mut self, request: grpc::Request<TaskCreate>) -> grpc::unary::ResponseFuture<Task, T::Future, T::ResponseBody>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<TaskCreate>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/scraping.ScraperTasksService/CreateTask");
            self.inner.unary(request, path)
        }

        /// A service that manages creation/destruction of scraping tasks
        /// 
        /// 'Scraper' will call those methods to initiate scraping and report it's progress
        /// and it's expected to be implemented by 'Importer'.
        pub fn yield_result<R>(&mut self, request: grpc::Request<TaskYield>) -> grpc::unary::ResponseFuture<(), T::Future, T::ResponseBody>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<TaskYield>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/scraping.ScraperTasksService/YieldResult");
            self.inner.unary(request, path)
        }

        /// A service that manages creation/destruction of scraping tasks
        /// 
        /// 'Scraper' will call those methods to initiate scraping and report it's progress
        /// and it's expected to be implemented by 'Importer'.
        pub fn complete_task<R>(&mut self, request: grpc::Request<TaskFinish>) -> grpc::unary::ResponseFuture<(), T::Future, T::ResponseBody>
        where T: grpc::GrpcService<R>,
              grpc::unary::Once<TaskFinish>: grpc::Encodable<R>,
        {
            let path = http::PathAndQuery::from_static("/scraping.ScraperTasksService/CompleteTask");
            self.inner.unary(request, path)
        }
    }
}

pub mod server {
    use ::tower_grpc::codegen::server::*;
    use super::{ScrapeIntent, ScrapeIntentResult, TaskCreate, Task, TaskYield, TaskFinish};

    // Redefine the try_ready macro so that it doesn't need to be explicitly
    // imported by the user of this generated code.
    macro_rules! try_ready {
        ($e:expr) => (match $e {
            Ok(futures::Async::Ready(t)) => t,
            Ok(futures::Async::NotReady) => return Ok(futures::Async::NotReady),
            Err(e) => return Err(From::from(e)),
        })
    }

    /// A service to start scraping process
    /// 
    /// 'Scraper' should implement a server side of the service and
    /// something from the outside needs to trigger scraping process.
    pub trait ScraperService: Clone {
        type StartScrapingFuture: futures::Future<Item = grpc::Response<ScrapeIntentResult>, Error = grpc::Status>;

        /// Starts web scraping and returns result of the operation when finished
        fn start_scraping(&mut self, request: grpc::Request<ScrapeIntent>) -> Self::StartScrapingFuture;
    }

    #[derive(Debug, Clone)]
    pub struct ScraperServiceServer<T> {
        scraper_service: T,
    }

    impl<T> ScraperServiceServer<T>
    where T: ScraperService,
    {
        pub fn new(scraper_service: T) -> Self {
            Self { scraper_service }
        }
    }

    impl<T> tower::Service<http::Request<grpc::BoxBody>> for ScraperServiceServer<T>
    where T: ScraperService,
    {
        type Response = http::Response<scraper_service::ResponseBody<T>>;
        type Error = grpc::Never;
        type Future = scraper_service::ResponseFuture<T>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(().into())
        }

        fn call(&mut self, request: http::Request<grpc::BoxBody>) -> Self::Future {
            use self::scraper_service::Kind::*;

            match request.uri().path() {
                "/scraping.ScraperService/StartScraping" => {
                    let service = scraper_service::methods::StartScraping(self.scraper_service.clone());
                    let response = grpc::unary(service, request);
                    scraper_service::ResponseFuture { kind: StartScraping(response) }
                }
                _ => {
                    scraper_service::ResponseFuture { kind: __Generated__Unimplemented(grpc::unimplemented(format!("unknown service: {:?}", request.uri().path()))) }
                }
            }
        }
    }

    impl<T> tower::Service<()> for ScraperServiceServer<T>
    where T: ScraperService,
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

    impl<T> tower::Service<http::Request<tower_hyper::Body>> for ScraperServiceServer<T>
    where T: ScraperService,
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

    pub mod scraper_service {
        use ::tower_grpc::codegen::server::*;
        use super::ScraperService;
        use super::super::ScrapeIntent;

        pub struct ResponseFuture<T>
        where T: ScraperService,
        {
            pub(super) kind: Kind<
                // StartScraping
                grpc::unary::ResponseFuture<methods::StartScraping<T>, grpc::BoxBody, ScrapeIntent>,
                // A generated catch-all for unimplemented service calls
                grpc::unimplemented::ResponseFuture,
            >,
        }

        impl<T> futures::Future for ResponseFuture<T>
        where T: ScraperService,
        {
            type Item = http::Response<ResponseBody<T>>;
            type Error = grpc::Never;

            fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    StartScraping(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: StartScraping(body) }
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
        where T: ScraperService,
        {
            pub(super) kind: Kind<
                // StartScraping
                grpc::Encode<grpc::unary::Once<<methods::StartScraping<T> as grpc::UnaryService<ScrapeIntent>>::Response>>,
                // A generated catch-all for unimplemented service calls
                (),
            >,
        }

        impl<T> tower::HttpBody for ResponseBody<T>
        where T: ScraperService,
        {
            type Data = <grpc::BoxBody as grpc::Body>::Data;
            type Error = grpc::Status;

            fn is_end_stream(&self) -> bool {
                use self::Kind::*;

                match self.kind {
                    StartScraping(ref v) => v.is_end_stream(),
                    __Generated__Unimplemented(_) => true,
                }
            }

            fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    StartScraping(ref mut v) => v.poll_data(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }

            fn poll_trailers(&mut self) -> futures::Poll<Option<http::HeaderMap>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    StartScraping(ref mut v) => v.poll_trailers(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }
        }

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub(super) enum Kind<StartScraping, __Generated__Unimplemented> {
            StartScraping(StartScraping),
            __Generated__Unimplemented(__Generated__Unimplemented),
        }

        pub mod methods {
            use ::tower_grpc::codegen::server::*;
            use super::super::{ScraperService, ScrapeIntent, ScrapeIntentResult};

            pub struct StartScraping<T>(pub T);

            impl<T> tower::Service<grpc::Request<ScrapeIntent>> for StartScraping<T>
            where T: ScraperService,
            {
                type Response = grpc::Response<ScrapeIntentResult>;
                type Error = grpc::Status;
                type Future = T::StartScrapingFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<ScrapeIntent>) -> Self::Future {
                    self.0.start_scraping(request)
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

    /// A service that manages creation/destruction of scraping tasks
    /// 
    /// 'Scraper' will call those methods to initiate scraping and report it's progress
    /// and it's expected to be implemented by 'Importer'.
    pub trait ScraperTasksService: Clone {
        type CreateTaskFuture: futures::Future<Item = grpc::Response<Task>, Error = grpc::Status>;
        type YieldResultFuture: futures::Future<Item = grpc::Response<()>, Error = grpc::Status>;
        type CompleteTaskFuture: futures::Future<Item = grpc::Response<()>, Error = grpc::Status>;

        /// Creates new scraping task and returns info about target to scrape
        fn create_task(&mut self, request: grpc::Request<TaskCreate>) -> Self::CreateTaskFuture;

        /// Reports that an atomic piece of data has been scraped
        fn yield_result(&mut self, request: grpc::Request<TaskYield>) -> Self::YieldResultFuture;

        /// Reports that scraping has finished and no more work will be done
        fn complete_task(&mut self, request: grpc::Request<TaskFinish>) -> Self::CompleteTaskFuture;
    }

    #[derive(Debug, Clone)]
    pub struct ScraperTasksServiceServer<T> {
        scraper_tasks_service: T,
    }

    impl<T> ScraperTasksServiceServer<T>
    where T: ScraperTasksService,
    {
        pub fn new(scraper_tasks_service: T) -> Self {
            Self { scraper_tasks_service }
        }
    }

    impl<T> tower::Service<http::Request<grpc::BoxBody>> for ScraperTasksServiceServer<T>
    where T: ScraperTasksService,
    {
        type Response = http::Response<scraper_tasks_service::ResponseBody<T>>;
        type Error = grpc::Never;
        type Future = scraper_tasks_service::ResponseFuture<T>;

        fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
            Ok(().into())
        }

        fn call(&mut self, request: http::Request<grpc::BoxBody>) -> Self::Future {
            use self::scraper_tasks_service::Kind::*;

            match request.uri().path() {
                "/scraping.ScraperTasksService/CreateTask" => {
                    let service = scraper_tasks_service::methods::CreateTask(self.scraper_tasks_service.clone());
                    let response = grpc::unary(service, request);
                    scraper_tasks_service::ResponseFuture { kind: CreateTask(response) }
                }
                "/scraping.ScraperTasksService/YieldResult" => {
                    let service = scraper_tasks_service::methods::YieldResult(self.scraper_tasks_service.clone());
                    let response = grpc::unary(service, request);
                    scraper_tasks_service::ResponseFuture { kind: YieldResult(response) }
                }
                "/scraping.ScraperTasksService/CompleteTask" => {
                    let service = scraper_tasks_service::methods::CompleteTask(self.scraper_tasks_service.clone());
                    let response = grpc::unary(service, request);
                    scraper_tasks_service::ResponseFuture { kind: CompleteTask(response) }
                }
                _ => {
                    scraper_tasks_service::ResponseFuture { kind: __Generated__Unimplemented(grpc::unimplemented(format!("unknown service: {:?}", request.uri().path()))) }
                }
            }
        }
    }

    impl<T> tower::Service<()> for ScraperTasksServiceServer<T>
    where T: ScraperTasksService,
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

    impl<T> tower::Service<http::Request<tower_hyper::Body>> for ScraperTasksServiceServer<T>
    where T: ScraperTasksService,
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

    pub mod scraper_tasks_service {
        use ::tower_grpc::codegen::server::*;
        use super::ScraperTasksService;
        use super::super::{TaskCreate, TaskYield, TaskFinish};

        pub struct ResponseFuture<T>
        where T: ScraperTasksService,
        {
            pub(super) kind: Kind<
                // CreateTask
                grpc::unary::ResponseFuture<methods::CreateTask<T>, grpc::BoxBody, TaskCreate>,
                // YieldResult
                grpc::unary::ResponseFuture<methods::YieldResult<T>, grpc::BoxBody, TaskYield>,
                // CompleteTask
                grpc::unary::ResponseFuture<methods::CompleteTask<T>, grpc::BoxBody, TaskFinish>,
                // A generated catch-all for unimplemented service calls
                grpc::unimplemented::ResponseFuture,
            >,
        }

        impl<T> futures::Future for ResponseFuture<T>
        where T: ScraperTasksService,
        {
            type Item = http::Response<ResponseBody<T>>;
            type Error = grpc::Never;

            fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    CreateTask(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: CreateTask(body) }
                        });
                        Ok(response.into())
                    }
                    YieldResult(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: YieldResult(body) }
                        });
                        Ok(response.into())
                    }
                    CompleteTask(ref mut fut) => {
                        let response = try_ready!(fut.poll());
                        let response = response.map(|body| {
                            ResponseBody { kind: CompleteTask(body) }
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
        where T: ScraperTasksService,
        {
            pub(super) kind: Kind<
                // CreateTask
                grpc::Encode<grpc::unary::Once<<methods::CreateTask<T> as grpc::UnaryService<TaskCreate>>::Response>>,
                // YieldResult
                grpc::Encode<grpc::unary::Once<<methods::YieldResult<T> as grpc::UnaryService<TaskYield>>::Response>>,
                // CompleteTask
                grpc::Encode<grpc::unary::Once<<methods::CompleteTask<T> as grpc::UnaryService<TaskFinish>>::Response>>,
                // A generated catch-all for unimplemented service calls
                (),
            >,
        }

        impl<T> tower::HttpBody for ResponseBody<T>
        where T: ScraperTasksService,
        {
            type Data = <grpc::BoxBody as grpc::Body>::Data;
            type Error = grpc::Status;

            fn is_end_stream(&self) -> bool {
                use self::Kind::*;

                match self.kind {
                    CreateTask(ref v) => v.is_end_stream(),
                    YieldResult(ref v) => v.is_end_stream(),
                    CompleteTask(ref v) => v.is_end_stream(),
                    __Generated__Unimplemented(_) => true,
                }
            }

            fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    CreateTask(ref mut v) => v.poll_data(),
                    YieldResult(ref mut v) => v.poll_data(),
                    CompleteTask(ref mut v) => v.poll_data(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }

            fn poll_trailers(&mut self) -> futures::Poll<Option<http::HeaderMap>, Self::Error> {
                use self::Kind::*;

                match self.kind {
                    CreateTask(ref mut v) => v.poll_trailers(),
                    YieldResult(ref mut v) => v.poll_trailers(),
                    CompleteTask(ref mut v) => v.poll_trailers(),
                    __Generated__Unimplemented(_) => Ok(None.into()),
                }
            }
        }

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub(super) enum Kind<CreateTask, YieldResult, CompleteTask, __Generated__Unimplemented> {
            CreateTask(CreateTask),
            YieldResult(YieldResult),
            CompleteTask(CompleteTask),
            __Generated__Unimplemented(__Generated__Unimplemented),
        }

        pub mod methods {
            use ::tower_grpc::codegen::server::*;
            use super::super::{ScraperTasksService, TaskCreate, Task, TaskYield, TaskFinish};

            pub struct CreateTask<T>(pub T);

            impl<T> tower::Service<grpc::Request<TaskCreate>> for CreateTask<T>
            where T: ScraperTasksService,
            {
                type Response = grpc::Response<Task>;
                type Error = grpc::Status;
                type Future = T::CreateTaskFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<TaskCreate>) -> Self::Future {
                    self.0.create_task(request)
                }
            }

            pub struct YieldResult<T>(pub T);

            impl<T> tower::Service<grpc::Request<TaskYield>> for YieldResult<T>
            where T: ScraperTasksService,
            {
                type Response = grpc::Response<()>;
                type Error = grpc::Status;
                type Future = T::YieldResultFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<TaskYield>) -> Self::Future {
                    self.0.yield_result(request)
                }
            }

            pub struct CompleteTask<T>(pub T);

            impl<T> tower::Service<grpc::Request<TaskFinish>> for CompleteTask<T>
            where T: ScraperTasksService,
            {
                type Response = grpc::Response<()>;
                type Error = grpc::Status;
                type Future = T::CompleteTaskFuture;

                fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
                    Ok(futures::Async::Ready(()))
                }

                fn call(&mut self, request: grpc::Request<TaskFinish>) -> Self::Future {
                    self.0.complete_task(request)
                }
            }
        }
    }
}
