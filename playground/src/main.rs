#![deny(unused_imports)]
#![deny(unused_mut)]
#![deny(dead_code)]
/*
use tokio::time::{timeout_at, Duration, Instant};
use tokio::time;
use futures::future::join_all;
use hyper::{Client, Request, Body};
use hyper::client::HttpConnector;
use hyper::client::connect::dns::GaiResolver;
use hyper::body::aggregate;
use hyper_tls::HttpsConnector;
use native_tls::TlsConnector;
use http::response::Parts;
use bytes::Buf;
use anyhow::Result;
use thiserror::Error;

pub type HttpsClient = Client<HttpsConnector<HttpConnector<GaiResolver>>, Body>;
pub type Outcome = Result<(Parts, Box<dyn Buf>), RequestError>;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("the request failed: {0:?}")]
    RequestFailure(hyper::Error),
    #[error("failure during response: {0:?}")]
    ResponseFailure(hyper::Error),
    #[error("timeout elapsed")]
    ResponseTimeout(time::Elapsed),
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let http_client = provide_https_client();

    let deadline = Instant::now() + Duration::from_millis(950);

    let mut out_requests = vec![];
    out_requests.push(Request::builder()
        .method("GET").uri("http://127.0.0.1:9999/").body(Body::empty()).unwrap()); // error
    out_requests.push(Request::builder()
        .method("GET").uri("http://sol.gligor.at:8080/").body(Body::empty()).unwrap());
    out_requests.push(Request::builder()
        .method("GET").uri("https://www.google.com/").body(Body::empty()).unwrap());
    out_requests.push(Request::builder()
        .method("GET").uri("https://eversign.com/").body(Body::empty()).unwrap()); // around 1 sec
    out_requests.push(Request::builder()
        .method("GET").uri("https://www.facebook.com/").body(Body::empty()).unwrap()); // 302

    // ---------------------------------------------------------------------

    let start_time = Instant::now();
    let outcomes = execute_requests(&http_client, out_requests, deadline.into()).await;

    for outcome in outcomes.iter() {
        match outcome {
            Err(RequestError::RequestFailure(e)) => println!("request failure: {}", e),
            Err(RequestError::ResponseFailure(e)) => println!("response failure: {}", e),
            Err(RequestError::ResponseTimeout(e)) => println!("response timeout: {}", e),
            Ok((head, body)) => println!("success: {}, {} bytes", head.status, body.remaining()),
        }
    }
    let total_time = Instant::now().duration_since(start_time);
    println!("duration: {}ms", total_time.as_millis());

    println!();
    println!("playground finished");
}

fn provide_https_client() -> HttpsClient {
    let http_connector = {
        // XXX TokioThreadpoolGaiResolver seems to be broken in hyper-0.13.0-alpha.4
        //let mut http_connector = HttpConnector::new_with_resolver(TokioThreadpoolGaiResolver::new());
        let mut http_connector = HttpConnector::new_with_resolver(GaiResolver::new());
        http_connector.enforce_http(false);
        http_connector
    };
    let tls_connector = TlsConnector::new().unwrap();
    let https_connector = HttpsConnector::from((http_connector, tls_connector.into()));

    Client::builder().build::<_, Body>(https_connector)
}

async fn execute_requests(client: &HttpsClient, mut requests: Vec<Request<Body>>,
                          deadline: Instant) -> Vec<Outcome>
{
    let responses_with_timeout = requests
        .drain(..)
        .map(|req| self::execute_request(client, req, deadline))
        .collect::<Vec<_>>();

    join_all(responses_with_timeout).await
}

async fn execute_request(client: &HttpsClient, request: Request<Body>, deadline: Instant)
    -> Outcome
{
    let timeout_future = timeout_at(deadline, async {
        let resp = client.request(request).await
            .map_err(|e| RequestError::RequestFailure(e))?;

        let (parts, body_stream) = resp.into_parts();

        let body = aggregate(body_stream).await
            .map_err(|e| RequestError::ResponseFailure(e))?;

        Ok((parts, Box::new(body) as Box<dyn Buf>))
    });

    timeout_future.await
        .map_err(|e| RequestError::ResponseTimeout(e))?
}
*/
