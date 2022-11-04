use async_trait::async_trait;
use anyhow::{Result, Context};
use http::Response;
use hyper::{Client, Request, Body};
use hyper::client::HttpConnector;
use hyper::client::connect::dns::GaiResolver;
use hyper_tls::HttpsConnector;
use native_tls::TlsConnector;

// the purpose of this trait is to decouple dependent code from the implementation and allow mocking
#[async_trait]
pub trait HttpClient {
    async fn request(&self, req: Request<Body>) -> Result<Response<Body>>;
}

// the purpose of this type is to hide the dependency on hyper::Client from the rest of the code
// XXX as long as we expose Body, Parts, Response and Request, the job is not yet done
#[derive(Clone)]
pub struct OctoplexHttpClient {
    inner: Client<HttpsConnector<HttpConnector<GaiResolver>>, Body>
}

#[async_trait]
impl HttpClient for OctoplexHttpClient {
    async fn request(&self, req: Request<Body>) -> Result<Response<Body>> {
        self.inner.request(req).await
            .map_err(|e| e.into())
    }
}

pub fn make_hyper_client() -> Result<OctoplexHttpClient> {
    let http_connector = {
        // XXX TokioThreadpoolGaiResolver seems to be broken in hyper-0.13.0-alpha.4
        //let mut http_connector = HttpConnector::new_with_resolver(TokioThreadpoolGaiResolver::new());
        let mut http_connector = HttpConnector::new_with_resolver(GaiResolver::new());
        http_connector.enforce_http(false);
        http_connector
    };
    let tls_connector = TlsConnector::new().context("cannot create TlsConnector")?;
    let https_connector = HttpsConnector::from((http_connector, tls_connector.into()));
    let inner = Client::builder().build(https_connector);

    Ok(OctoplexHttpClient { inner })
}

#[cfg(test)]
pub(crate) mod tests {
    use anyhow::Result;
    use async_trait::async_trait;
    use hyper::{Request, Response, Body};
    use mockall::*;
    use mockall::predicate::*;
    use tokio::time::{sleep, Duration};

    use crate::http_client::{HttpClient};

    pub const MOCK_REQUEST_DURATION: Duration = Duration::from_millis(50);

    mock! {
        pub HttpClient {
            pub fn request(&self, req: Request<Body>) -> Result<Response<Body>> {}
        }

        impl Clone for HttpClient {
            fn clone(&self) -> Self;
        }
    }

    #[async_trait]
    impl HttpClient for MockHttpClient {
        async fn request(&self, req: Request<Body>) -> Result<Response<Body>> {
            sleep(MOCK_REQUEST_DURATION).await;

            self.request(req)
        }
    }
}
