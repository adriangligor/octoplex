use anyhow::Error as AnyError;
use tokio::time::{timeout_at, Duration, Elapsed, Instant};
use futures::future::join_all;
use humantime::format_duration;
use anyhow::Result;
use thiserror::Error;

// XXX these dependencies have to be removed, we should only depend on http_client
use hyper::{Request, Body};
use hyper::body::to_bytes;
use http::response::Parts;

use crate::api::{OctoplexRequest, OctoplexResponse, SingleHttpResponse, SingleOutcome, SingleHttpFailure};
use crate::http_client::{HttpClient, OctoplexHttpClient};

pub type Multiplexer = GenericMultiplexer<OctoplexHttpClient>;
type ValidationOutcome = Result<OctoplexRequest, ValidationError>;
type RequestOutcome = Result<(Duration, Parts, String), RequestError>;

#[derive(Error, Debug)]
enum ValidationError {
    #[error("timeout may not be more than {}", format_duration(*.0))]
    MaximumTimeoutExceeded(Duration),
    #[error("there must be at least one request in the batch")]
    EmptyBatchRequested,
    #[error("there may not be more than {0} requests in the batch")]
    MaximumBatchSizeExceeded(usize),
}

#[derive(Error, Debug)]
enum RequestError {
    #[error("the request was invalid: {error}")]
    RequestInvalid { error: AnyError },
    #[error("the request failed: {error}")]
    RequestFailure { error: AnyError, duration: Duration },
    #[error("failure during response: {error}")]
    ResponseFailure { error: AnyError, duration: Duration },
    #[error("timeout elapsed")]
    ResponseTimeout { error: Elapsed, duration: Duration },
}

enum ValidatedRequest {
    ValidRequest(Request<Body>),
    InvalidRequest(AnyError),
}

const MAX_REQUEST_DURATION: Duration = Duration::from_millis(60 * 60 * 1_000); // XXX config
const MAX_BATCH_SIZE: usize = 50; // XXX config

#[derive(Clone)]
pub struct GenericMultiplexer<C>
    where C: HttpClient + Clone + Send + Sync
{
    http_client: C // client is cheap to clone and retains shared state (pool, connector)
    // XXX dns cache, metrics, etc
}

impl<C> GenericMultiplexer<C>
    where C: HttpClient + Clone + Send + Sync
{
    pub fn new(http_client: C) -> Self {
        GenericMultiplexer {
            http_client
        }
    }

    pub async fn handle(&self, batch: OctoplexRequest) -> Result<OctoplexResponse> {
        let batch = Self::validate_request(batch)?;

        let deadline = Instant::now() + batch.timeout_msec;
        let out_requests = Self::build_out_requests(batch);

        let mut outcomes = self.execute_requests(out_requests, deadline).await;

        let mut responses = vec![];
        for outcome in outcomes.drain(..) {
            let response = match outcome {
                Err(RequestError::RequestInvalid { error }) =>
                    SingleOutcome::Failure(SingleHttpFailure {
                        error: error.to_string(),
                        duration_msec: Duration::from_millis(0),
                    }),
                Err(RequestError::RequestFailure { error, duration }) =>
                    SingleOutcome::Failure(SingleHttpFailure {
                        error: error.to_string(),
                        duration_msec: duration,
                    }),
                Err(RequestError::ResponseFailure { error, duration }) =>
                    SingleOutcome::Failure(SingleHttpFailure {
                        error: error.to_string(),
                        duration_msec: duration,
                    }),
                Err(RequestError::ResponseTimeout { error, duration }) =>
                    SingleOutcome::Failure(SingleHttpFailure {
                        error: error.to_string(),
                        duration_msec: duration,
                    }),
                Ok((req_duration, head, body_bytes)) =>
                    SingleOutcome::Success(SingleHttpResponse {
                        headers: head.headers.into(),
                        status: head.status.as_u16(),
                        content: Some(body_bytes),
                        duration_msec: req_duration,
                    }),
            };
            responses.push(response);
        }

        Ok(OctoplexResponse {
            responses
        })
    }

    fn validate_request(batch: OctoplexRequest) -> ValidationOutcome {
        if batch.timeout_msec > MAX_REQUEST_DURATION {
            return Err(ValidationError::MaximumTimeoutExceeded(MAX_REQUEST_DURATION));
        }

        if batch.requests.len() == 0 {
            return Err(ValidationError::EmptyBatchRequested);
        }

        if batch.requests.len() > MAX_BATCH_SIZE {
            return Err(ValidationError::MaximumBatchSizeExceeded(MAX_BATCH_SIZE));
        }

        Ok(batch)
    }

    fn build_out_requests(batch: OctoplexRequest) -> Vec<ValidatedRequest> {
        let mut out_reqs = Vec::new();

        for http_req in batch.requests {
            let mut out_req_builder = Request::builder()
                .method(http_req.method.as_ref())
                .uri(http_req.uri);

            for (name, value) in &http_req.headers {
                out_req_builder = out_req_builder.header(name, value);
            }

            let req_body = match http_req.body {
                Some(body) => Body::from(body),
                None => Body::empty(),
            };

            let out_req = out_req_builder.body(req_body)
                .map(|req| ValidatedRequest::ValidRequest(req))
                .unwrap_or_else(|err| ValidatedRequest::InvalidRequest(err.into()));

            out_reqs.push(out_req);
        }

        out_reqs
    }

    async fn execute_requests(&self, mut requests: Vec<ValidatedRequest>,
                              deadline: Instant) -> Vec<RequestOutcome>
    {
        // XXX a low timeout will interrupt establishing and keeping a keep-alive connection, which
        // would otherwise speed up subsequent requests
        let responses_with_timeout = requests
            .drain(..)
            .map(|req| self.execute_request(req, deadline))
            .collect::<Vec<_>>();

        join_all(responses_with_timeout).await
    }

    async fn execute_request(&self, request: ValidatedRequest, deadline: Instant)
                             -> RequestOutcome
    {
        let request = match request {
            ValidatedRequest::ValidRequest(req) => req,
            ValidatedRequest::InvalidRequest(err) => return Err(RequestError::RequestInvalid { error: err }),
        };

        let timeout_start_time = Instant::now();

        let timeout_future = timeout_at(deadline, async {
            let start_time = Instant::now();
            let resp = self.http_client.request(request).await
                .map_err(|error| {
                    let duration = Instant::now().saturating_duration_since(timeout_start_time);

                    RequestError::RequestFailure { error, duration }
                })?;

            let (parts, body_stream) = resp.into_parts();

            // XXX impl Buf is not Send (disabled code), we have to clone/own the data :(
            //use hyper::body::aggregate;
            //use bytes::Bytes;
            //let body = aggregate(body_stream).await
            //    .map_err(|e| RequestError::ResponseFailure(e))?;

            //Ok((parts, Box::new(body) as Box<dyn Buf>))

            let body_bytes = to_bytes(body_stream).await // XXX clone :(
                .map(|b| String::from(String::from_utf8_lossy(&b))) // XXX copy :(
                .map_err(|error| {
                    let duration = Instant::now().saturating_duration_since(start_time);

                    RequestError::ResponseFailure { error: error.into(), duration }
                })?;
            // XXX what is needed here is something that can be serialized to JSON with Serde, but is
            // backed by a Buf (non-contiguous), this way we could achieve zerocopy
            // Vecs are backed by a contiguous buffer

            let duration = Instant::now().saturating_duration_since(start_time);

            Ok((duration, parts, body_bytes))
        });

        timeout_future.await
            .map_err(|error| {
                let duration = Instant::now().saturating_duration_since(timeout_start_time);

                RequestError::ResponseTimeout { error, duration }
            })?
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::{Context, Result};
    use hyper::{Response, Body};
    use thiserror::Error;

    use crate::http_client::tests::{MOCK_REQUEST_DURATION, MockHttpClient};
    use crate::multiplexer::{GenericMultiplexer};
    use crate::api::{OctoplexRequest, SingleHttpRequest, HttpMethod};

    #[derive(Error, Debug)]
    enum SimpleError {
        #[error("just some error")]
        SomeError
    }

    fn ok_response() -> Result<Response<Body>> {
        let json = "{}";

        Response::builder()
            .status(200)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(Body::from(json))
            .context("cannot build response")
    }

    fn err_response() -> Result<Response<Body>> {
        Err(SimpleError::SomeError.into())
    }

    fn google_request() -> SingleHttpRequest {
        SingleHttpRequest {
            method: HttpMethod::GET,
            uri: "https://www.google.com/".to_string(),
            headers: Default::default(),
            body: None
        }
    }

    #[tokio::test]
    async fn rejects_excessive_timeout() {
        let client = MockHttpClient::new();

        let batch = OctoplexRequest {
            timeout_msec: Duration::from_millis(5_000_000),
            requests: vec![google_request()],
        };

        let result = GenericMultiplexer::new(client)
            .handle(batch).await;

        assert!(result.is_err(), "expected Err, got result = {:?}", result);
    }

    #[tokio::test]
    async fn rejects_empty_batch() {
        let client = MockHttpClient::new();

        let batch = OctoplexRequest {
            timeout_msec: MOCK_REQUEST_DURATION * 2,
            requests: vec![],
        };

        let result = GenericMultiplexer::new(client)
            .handle(batch).await;

        assert!(result.is_err(), "expected Err, got result = {:?}", result);
    }

    #[tokio::test]
    async fn rejects_oversized_batch() {
        let client = MockHttpClient::new();

        let mut requests = vec![];
        for _ in 0..75 {
            requests.push(google_request());
        }

        let batch = OctoplexRequest {
            timeout_msec: MOCK_REQUEST_DURATION * 2,
            requests,
        };

        let result = GenericMultiplexer::new(client)
            .handle(batch).await;

        assert!(result.is_err(), "expected Err, got result = {:?}", result);
    }

    #[tokio::test]
    async fn uses_http_client() {
        let mut client = MockHttpClient::new();
        client.expect_request().returning(|_req| ok_response());

        let batch = OctoplexRequest {
            timeout_msec: MOCK_REQUEST_DURATION * 2,
            requests: vec![google_request()],
        };

        let result = GenericMultiplexer::new(client)
            .handle(batch).await;

        assert!(result.is_ok(), "expected Ok, got result = {:?}", result);
        assert_eq!(result.as_ref().unwrap().responses.len(), 1);
        assert_eq!(result.as_ref().unwrap().responses[0].as_ref(), "Success");
    }

    #[tokio::test]
    async fn handles_timeout() {
        let mut client = MockHttpClient::new();
        client.expect_request().returning(|_req| ok_response());

        let batch = OctoplexRequest {
            timeout_msec: MOCK_REQUEST_DURATION / 2,
            requests: vec![google_request()],
        };

        let result = GenericMultiplexer::new(client)
            .handle(batch).await;

        assert!(result.is_ok(), "expected Ok, got result = {:?}", result);
        assert_eq!(result.as_ref().unwrap().responses.len(), 1);
        assert_eq!(result.as_ref().unwrap().responses[0].as_ref(), "Failure");
    }

    #[tokio::test]
    async fn handles_http_error() {
        let mut client = MockHttpClient::new();
        client.expect_request().returning(|_req| err_response());

        let batch = OctoplexRequest {
            timeout_msec: MOCK_REQUEST_DURATION * 2,
            requests: vec![google_request()],
        };

        let result = GenericMultiplexer::new(client)
            .handle(batch).await;

        assert!(result.is_ok(), "expected Ok, got result = {:?}", result);
        assert_eq!(result.as_ref().unwrap().responses.len(), 1);
        assert_eq!(result.as_ref().unwrap().responses[0].as_ref(), "Failure");
    }
}
