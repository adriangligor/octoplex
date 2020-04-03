mod api;
mod http_client;
mod http_server;
mod multiplexer;

extern crate strum;
#[macro_use]
extern crate serde_derive;

use std::net::SocketAddr;

use anyhow::Result;

use crate::multiplexer::Multiplexer;
use crate::http_server::launch_http_server;
use crate::http_client::make_hyper_client;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let http_client = make_hyper_client()?;
    let multiplexer = Multiplexer::new(http_client);
    let http_server = launch_http_server(&addr, &multiplexer);

    println!("Listening on http://{}", addr);

    // XXX here we can join another server, for example gRPC (maybe get inspired by tower)
    Ok(http_server.await?)
}
