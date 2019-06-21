use futures::future::err;
use futures::prelude::*;
use prost::Message;
use reqwest::r#async::{Client, ClientBuilder};

pub use prost::EncodeError as ProtoEncodeError;
pub use reqwest::Error as ReqwestError;

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::time::Duration;

use super::DumpImportError;
use crate::proto::scheduler::intent::*;

pub fn responder(
    import_result: Result<HashSet<i32>, DumpImportError>,
    intent: ImportIntent,
) -> impl Future<Item = (), Error = ProtoSenderError> + Send {
    let client = ClientBuilder::new()
        .gzip(true)
        .connect_timeout(Duration::new(60, 0))
        .build();

    futures::future::result(client)
        .from_err()
        .and_then(|client| ImportResponder::new(import_result, intent, client).respond())
}

/// A struct that sends request to `ImportIntent` callback url with import task result
pub struct ImportResponder<S> {
    /// Import task result
    import_result: Result<HashSet<i32>, DumpImportError>,

    /// An intent that triggered import task
    intent: ImportIntent,

    /// Actual request sender that support protobuf as request body
    proto_sender: S,
}

impl<S: ProtoSender> ImportResponder<S> {
    /// Creates new struct instance
    pub fn new(
        import_result: Result<HashSet<i32>, DumpImportError>,
        intent: ImportIntent,
        proto_sender: S,
    ) -> Self {
        Self {
            import_result,
            intent,
            proto_sender,
        }
    }

    /// Sends response to `ImportIntent` with import result
    pub fn respond(self) -> impl Future<Item = (), Error = ProtoSenderError> {
        let ImportIntent {
            id, callback_url, ..
        } = self.intent;

        let response = match self.import_result {
            Ok(skipped_ids) => ImportIntentResult {
                id,
                skipped_ids: skipped_ids.into_iter().collect(),
                succeeded: true,
                error_description: String::new(),
            },
            Err(e) => ImportIntentResult {
                id,
                skipped_ids: Vec::new(),
                succeeded: false,
                error_description: e.to_string(),
            },
        };

        self.proto_sender.send_proto(response, &callback_url)
    }
}

/// Trait that allows sending HTTP(S) requests with protobuf as body
pub trait ProtoSender: Send + 'static {
    /// Sends request to specified `url` with `message` as it's body
    fn send_proto<M: Message>(
        &self,
        message: M,
        url: &str,
    ) -> Box<dyn Future<Item = (), Error = ProtoSenderError> + Send>;
}

impl ProtoSender for Client {
    fn send_proto<M: Message>(
        &self,
        message: M,
        url: &str,
    ) -> Box<dyn Future<Item = (), Error = ProtoSenderError> + Send + 'static> {
        let mut buf = Vec::new();
        match message.encode(&mut buf) {
            Err(e) => return Box::new(err(e.into())),
            _ => (),
        };

        let f = self
            .post(url)
            .body(buf)
            .send()
            .and_then(|_| Ok(()))
            .from_err();

        Box::new(f)
    }
}

/// An error that can happen during
#[derive(Debug)]
pub enum ProtoSenderError {
    /// Protobuf encoding error
    EncodingError(ProtoEncodeError),

    /// HTTP request error
    RequestError(Box<dyn Error + Send + 'static>),
}

impl fmt::Display for ProtoSenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ProtoSenderError::*;

        match self {
            EncodingError(e) => e.fmt(f),
            RequestError(e) => fmt::Display::fmt(e, f),
        }
    }
}

impl From<ProtoEncodeError> for ProtoSenderError {
    fn from(e: ProtoEncodeError) -> Self {
        ProtoSenderError::EncodingError(e)
    }
}

impl From<ReqwestError> for ProtoSenderError {
    fn from(e: ReqwestError) -> Self {
        ProtoSenderError::RequestError(Box::new(e))
    }
}
