# Octoplex

**HTTP request multiplexer**

Octoplex is a service that accepts a request containing a batch of HTTP requests, executes them all in parallel and returns all results. It handles errors, DNS resolver caching, TCP connection pooling and HTTP keep-alive and enforces deadlines/timeouts.

The main design goal is to be as efficient as possible by having an async architecture, using all CPU cores, locking as little as possible and avoid unnecessary memory copies.


## Get started

Just follow these simple steps:

1. Clone the repository, for example `git clone https://github.com/adriangligor/octoplex.git`
2. Start the Octoplex service via `cargo run --release`
3. Make a request  
```sh
curl --location --request POST 'http://localhost:8080/multiplex' \
     --header 'Content-Type: application/json' \
     --data-raw '{
       "timeout_msec": 250,
       "requests": [
         {
           "uri": "https://www.google.com/",
           "headers": {
             "X-Some-Header": "some value"
           }
         },
         {
           "method": "POST",
           "uri": "https://reqres.in/api/users",
           "headers": {
             "Content-Type": "application/json"
           },
           "body": "{\"name\":\"Bruce Campbell\",\"job\":\"Fake Shemp\"}"
         }
       ]
     }'
```

**Docker support**

For building a release image containing the Octoplex service, run:
`docker build -t adriangligor/octoplex:latest .`

For starting Octoplex, run:
`docker run -p 8080:8080 adriangligor/octoplex`. Now requests can be sent to `http://localhost:8080/multiplex`.


## Integration tests

Tests can be run as usual via `cargo test`. This includes integration tests, which require (and check for) mock services like WireMock (used as a target service for Octoplex) to be brought up and configured upfront. For this reason, a Docker Compose setup is provided as well (see below). When not using Docker Compose, follow these steps:

1. Start WireMock using the content of `extra/wiremock` as its root directory (see [--root-dir in the WireMock documentation](http://wiremock.org/docs/running-standalone/)). The provided Docker Compose support can also be used to start WireMock only.
2. Start the Octoplex service via `cargo run`
3. Run the tests via `cargo test`

Integration tests are locating the Octoplex and WireMock services by using the following environment variables:
- `TARGET_HOST` (default: "localhost"), the host running the Octoplex service
- `TARGET_PORT` (default: "8080"), the port the Octoplex service is listening on
- `WIREMOCK_HOST` (default: "localhost"), the host running WireMock
- `WIREMOCK_PORT` (default: "18080"), the port the WireMock service is listening on
- `WIREMOCK_PORT_SSL` (default: "18443"), the SSL port the WireMock service is listening on

**Docker Compose support**

In order to simplify the development of integration tests, a Docker Compose setup is provided. By running `docker-compose up -d octoplex` the service starts up via "cargo run" using the default profile, with source mapped into the container. When the source changes, a restart is enough to pick up the change. A WireMock container is also started.  
By running `docker-compose run tests`, the test suite is ran against the Octoplex container using WireMock.


## Roadmap items

- [ ] Introduce CLI arguments
- [ ] Introduce logging of key information (config, incoming requests, deadline violations, ...)
- [ ] Support logging to a log file
- [ ] Implement more unit and integration tests
- [ ] Support detaching as daemon
- [ ] Ensure DNS resolving and connection establishment in the background even after reaching batch timeout
- [ ] Ensure async DNS resolving, caching, respecting TTL and distributing evenly across multiple resolved IPs
- [ ] Ensure proper caching of TCP connections to resolved IPs, max request count or TTL per connection, growing and shrinking of connection pool
- [ ] Ensure detection and cleanup of stale connections
- [x] Create a good Dockerfile
- [ ] Support gRPC (or similar, with zero-copy capabilities) frontend in addition to HTTP frontend (split off frontends into modules or workspace crates)
- [ ] Collect statistics and make the available to clients
- [ ] Support TLS both on all frontends and outgoing
- [ ] Support outgoing gzip compression
- [ ] Ensure minimal overhead over actual outgoing requests
- [ ] Minimize incoming request size (e.g. when batched requests are very similar, like in OpenRTB auctions)
- [ ] Introduce config file
- [ ] Profile CPU and memory usage, allocations, etc., then optimise
