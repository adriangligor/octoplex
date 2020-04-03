pub mod api;

use std::collections::HashMap;
use std::env;

use serde_json::Value;
use reqwest::blocking::Client;

pub struct Config {
    pub target_host: String,
    pub target_port: u16,
    pub wiremock_host: String,
    pub wiremock_port: u16,
    pub wiremock_port_ssl: u16,
}

impl Config {
    pub fn build() -> Self {
        Config {
            target_host: env::var("TARGET_HOST")
                .unwrap_or("localhost".to_string()),
            target_port: env::var("TARGET_PORT")
                .unwrap_or("8080".to_string())
                .parse().expect("invalid target port"),
            wiremock_host: env::var("WIREMOCK_HOST")
                .unwrap_or("localhost".to_string()),
            wiremock_port: env::var("WIREMOCK_PORT")
                .unwrap_or("18080".to_string())
                .parse().expect("invalid WireMock port"),
            wiremock_port_ssl: env::var("WIREMOCK_PORT_SSL")
                .unwrap_or("18443".to_string())
                .parse().expect("invalid WireMock port"),
        }
    }
}

pub struct TestEnv {
    pub oc_base_url: String,
    pub wm_base_url: String,
    pub wm_base_url_ssl: String,
    pub http: Client,
    pub https: Client,
}

impl TestEnv {
    fn build(cfg: &Config) -> Self {
        TestEnv {
            oc_base_url: format!("http://{}:{}", cfg.target_host, cfg.target_port),
            wm_base_url: format!("http://{}:{}", cfg.wiremock_host, cfg.wiremock_port),
            wm_base_url_ssl: format!("https://{}:{}", cfg.wiremock_host, cfg.wiremock_port_ssl),
            http: reqwest::blocking::Client::builder()
                .build().unwrap(),
            https: reqwest::blocking::Client::builder()
                .danger_accept_invalid_certs(true)
                .build().unwrap(),
        }
    }
}

pub fn test_env() -> TestEnv {
    let cfg = Config::build();

    TestEnv::build(&cfg)
}

pub fn setup() {
    let test_env = test_env();

    // verify that infrastructure is up and running
    let oc_healthz = format!("{}/healthz", test_env.oc_base_url);
    test_env.http.get(&oc_healthz).send()
        .expect("octoplex target host unreachable")
        .json::<HashMap<String, bool>>()
        .expect("octoplex target host not healthy");
    let wm_admin = format!("{}/__admin", test_env.wm_base_url);
    test_env.http.get(&wm_admin).send()
        .expect("wiremock host unreachable")
        .json::<Value>()
        .expect("wiremock host not healthy");
    let wm_admin_ssl = format!("{}/__admin", test_env.wm_base_url_ssl);
    test_env.https.get(&wm_admin_ssl).send()
        .expect("wiremock host (ssl) unreachable")
        .json::<Value>()
        .expect("wiremock host (ssl) not healthy");
}
