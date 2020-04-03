#[macro_use]
extern crate serde_derive;

mod common;

use std::collections::HashMap;
use std::time::Duration;

use crate::common::api::{OctoplexRequest, OctoplexResponse, SingleHttpRequest, HttpMethod, SingleOutcome};

#[test]
fn handles_single_ok_request() {
    common::setup();
    let test_env = common::test_env();

    let batch = OctoplexRequest {
        timeout_msec: Duration::from_millis(500),
        requests: vec![
            SingleHttpRequest {
                method: HttpMethod::GET,
                uri: format!("{}/hello", test_env.wm_base_url),
                headers: HashMap::new(),
                body: None,
            }
        ],
    };

    let oc_multiplex = format!("{}/multiplex", test_env.oc_base_url);
    let oc_resp = test_env.http.post(&oc_multiplex)
        .json(&batch).send()
        .expect("octoplex target host unreachable")
        .json::<OctoplexResponse>()
        .expect("invalid octoplex response");

    let resp = match oc_resp.responses.get(0).expect("expected a response") {
        SingleOutcome::Success(resp) => resp,
        _ => panic!("expected a success response"),
    };

    assert_eq!(resp.status, 200);
    assert_eq!(resp.content.as_ref().unwrap(), "Hello world!");
}
