use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::time::Duration;

use http::{HeaderMap, HeaderValue};
use serde::{Serialize, Serializer};
use serde::ser::SerializeMap;
use strum_macros::AsRefStr;

// XXX using String probably causes a copy, use Cow or &str

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OctoplexRequest {
    #[serde(with = "serde_millis")]
    pub timeout_msec: Duration,
    pub requests: Vec<SingleHttpRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleHttpRequest {
    #[serde(default)]
    pub method: HttpMethod,
    pub uri: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OctoplexResponse {
    pub responses: Vec<SingleOutcome>, // same order and count as requests!
}

#[derive(Debug, Serialize)]
pub struct OctoplexError {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub healthy: bool,
}

#[derive(Debug, AsRefStr, Serialize)]
pub enum SingleOutcome {
    Failure(SingleHttpFailure),
    Success(SingleHttpResponse),
}

impl PartialEq for SingleOutcome {
    fn eq(&self, other: &Self) -> bool {
        use SingleOutcome::*;

        match (self, other) {
            (Failure(_), Failure(_)) => true,
            (Success(_), Success(_)) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SingleHttpFailure {
    pub error: String,
    #[serde(with = "serde_millis")]
    pub duration_msec: Duration,
}

#[derive(Debug, Serialize)]
pub struct SingleHttpResponse {
    pub headers: Headers,
    pub status: u16,
    pub content: Option<String>,
    #[serde(with = "serde_millis")]
    pub duration_msec: Duration,
}

#[derive(Debug, AsRefStr, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::GET
    }
}

pub struct Headers(HeaderMap<HeaderValue>);

impl From<HeaderMap<HeaderValue>> for Headers {
    fn from(h: HeaderMap<HeaderValue>) -> Headers {
        Headers(h)
    }
}

impl Debug for Headers {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "(")?;
        self.0.iter()
            .for_each(|e| write!(f, "({:?}, {:?})", e.0, e.1).expect("cannot format string"));
        write!(f, ")")?;
        Ok(())
    }
}

impl Serialize for Headers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        // XXX a multimap does not translate well into a JSON object, as keys must be unique
        let mut map = serializer.serialize_map(None)?;
        for (k, v) in &self.0 {
            map.serialize_entry(k.as_str(), v.to_str().unwrap_or_default())?;
        }
        map.end()
    }
}
