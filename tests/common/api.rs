use std::collections::HashMap;
use std::time::Duration;

use strum_macros::AsRefStr;

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OctoplexRequest {
    #[serde(with = "serde_millis")]
    pub timeout_msec: Duration,
    pub requests: Vec<SingleHttpRequest>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SingleHttpRequest {
    #[serde(default)]
    pub method: HttpMethod,
    pub uri: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OctoplexResponse {
    pub responses: Vec<SingleOutcome>, // same order and count as requests!
}

#[derive(Debug, Deserialize)]
pub struct OctoplexError {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
}

#[derive(Debug, AsRefStr, Deserialize)]
pub enum SingleOutcome {
    Failure(SingleHttpFailure),
    Success(SingleHttpResponse),
}

#[derive(Debug, Deserialize)]
pub struct SingleHttpFailure {
    pub error: String,
    #[serde(with = "serde_millis")]
    pub duration_msec: Duration,
}

#[derive(Debug, Deserialize)]
pub struct SingleHttpResponse {
    pub headers: HashMap<String, String>,
    pub status: u16,
    pub content: Option<String>,
    #[serde(with = "serde_millis")]
    pub duration_msec: Duration,
}

#[allow(dead_code)]
#[derive(Debug, AsRefStr, Serialize)]
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
