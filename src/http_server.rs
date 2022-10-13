use std::net::SocketAddr;

use anyhow::{Context, Error, Result};
use hyper::{Body, Method, Request, Response, header};
use hyper::server::Server;
use hyper::body::aggregate;
use hyper::service::{service_fn, make_service_fn};
use hyper::server::conn::AddrStream;

use crate::api::{HealthResponse, OctoplexError};
use crate::multiplexer::Multiplexer;

pub async fn launch_http_server(addr: &SocketAddr, multi: &Multiplexer) -> Result<()>
{
    // XXX the nested blocks and repeated clones look horrible, but all this seems necessary
    // https://vorner.github.io/2020/04/13/hyper-traps.html
    let connection_handler = make_service_fn(|_conn: &AddrStream| {
        let multi = multi.clone();

        async {
            let request_handler = service_fn(move |req| {
                let multi = multi.clone();

                async move {
                    router(&multi, req).await
                }
            });

            Ok::<_, Error>(request_handler)
        }
    });

    let server = Server::try_bind(&addr)
        .with_context(|| format!("failed to bind HTTP server to {}", addr))?
        // serve() spawns threads, and requires 'static on all references in connection_handler
        .serve(connection_handler);

        Ok(server.await?)
}

async fn router(multi: &Multiplexer, req: Request<Body>) -> Result<Response<Body>> {
    let method = req.method();
    let uri_path = req.uri().path();

    match (method, uri_path) {
        (&Method::GET, "/") |
        (&Method::GET, "/healthz") => route_health_check().await,
        (&Method::POST, "/multiplex") => route_multiplex(multi, req).await,
        _ => route_not_found().await,
    }
}

async fn route_health_check() -> Result<Response<Body>> {
    let resp = HealthResponse {
        healthy: true
    };
    let resp_json = serde_json::to_string(&resp).context("cannot serialize")?;

    Response::builder()
        .status(200)
        .header("Content-Type", "application/json; charset=utf-8")
        .body(Body::from(resp_json))
        .context("cannot build response")
}

async fn route_multiplex(multi: &Multiplexer, req: Request<Body>) -> Result<Response<Body>> {
    // deserialize json body
    // XXX ensure zero-copy

    use bytes::Buf;

    // XXX is there a more idiomatic way to do this? we map the Err back to Ok!
    let entire_body = match aggregate(req).await {
        Ok(b) => b,
        Err(e) => return Ok(error_response(e)?),
    };

    let oct_req = match serde_json::from_reader(entire_body.reader()) {
        Ok(b) => b,
        Err(e) => return Ok(error_response(e)?),
    };

    let oct_resp = match multi.handle(oct_req).await {
        Ok(r) => r,
        Err(e) => return Ok(error_response(e)?),
    };

    let oct_resp_json = serde_json::to_string(&oct_resp).context("cannot serialize")?;

    Response::builder()
        .status(200)
        .header("Content-Type", "application/json; charset=utf-8")
        .body(Body::from(oct_resp_json))
        .context("cannot build response")
}

async fn route_not_found() -> Result<Response<Body>> {
    Response::builder()
        .status(404)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from("not found\n"))
        .context("cannot build response")
}

fn error_response(err: impl ToString) -> Result<Response<Body>> {
    let error = OctoplexError {
        error: err.to_string()
    };
    let error_json = serde_json::to_string(&error).context("cannot serialize")?;

    Response::builder()
        .status(400)
        .header("Content-Type", "application/json; charset=utf-8")
        .body(Body::from(error_json))
        .context("cannot build response")
}
