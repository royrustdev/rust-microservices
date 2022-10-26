# Hyper Server Benchmark

Benchmark with **wrk** tool

```
-c, --connections: total number of HTTP connections to keep open with each thread handling N = connections/threads

-d, --duration:    duration of the test, e.g. 2s, 2m, 2h

-t, --threads:     total number of threads to use
```

## source code v0.2

```rust
#![allow(deprecated)]

use futures::{future, Future};
use hyper::service::service_fn;
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use regex::Regex;
use slab::Slab;
use std::fmt;
use std::sync::{Arc, Mutex};

type UserId = u64;
struct UserData;

impl fmt::Display for UserData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{}")
    }
}

type UserDb = Arc<Mutex<Slab<UserData>>>;

const INDEX: &'static str = r#"
<!doctype html>
<html>
    <head>
        <title>hyper microservice</title>
    </head>
    <body>
        <h1>Microservices with Hyper</h1>
    </body>
</html>
"#;

// uses regex patterns to match path parameters
lazy_static! {
    static ref INDEX_PATH: Regex = Regex::new("^/(index\\.html?)?$").unwrap();
    static ref USER_PATH: Regex = Regex::new("^/user/((?P<user_id>\\d+?)/?)?$").unwrap();
    static ref USERS_PATH: Regex = Regex::new("^/users/?$").unwrap();
}

/// creates empty body with status code
fn response_with_code(status_code: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap()
}

/// handles services
fn service_handler(
    req: Request<Body>,
    user_db: &UserDb,
) -> impl Future<Item = Response<Body>, Error = Error> {
    let response = {
        let method = req.method();
        let path = req.uri().path();
        let mut users = user_db.lock().unwrap();

        if INDEX_PATH.is_match(path) {
            if method == &Method::GET {
                Response::new(INDEX.into())
            } else {
                response_with_code(StatusCode::METHOD_NOT_ALLOWED)
            }
        } else if USERS_PATH.is_match(path) {
            if method == &Method::GET {
                let list = users
                    .iter()
                    .map(|(id, _)| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                Response::new(list.into())
            } else {
                response_with_code(StatusCode::METHOD_NOT_ALLOWED)
            }
        } else if let Some(cap) = USER_PATH.captures(path) {
            let user_id = cap
                .name("user_id")
                .and_then(|m| m.parse::<UserId>().ok().map(|x| x as usize));

            match (method, user_id) {
                (&Method::POST, None) => {
                    let id = users.insert(UserData);
                    Response::new(id.to_string().into())
                }
                (&Method::POST, Some(_)) => response_with_code(StatusCode::BAD_REQUEST),
                (&Method::GET, Some(id)) => {
                    if let Some(data) = users.get(id) {
                        Response::new(data.to_string().into())
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                }
                (&Method::PUT, Some(id)) => {
                    if let Some(user) = users.get_mut(id) {
                        *user = UserData;
                        response_with_code(StatusCode::OK)
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                }
                (&Method::DELETE, Some(id)) => {
                    if users.contains(id) {
                        users.remove(id);
                        response_with_code(StatusCode::OK)
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                }
                _ => response_with_code(StatusCode::METHOD_NOT_ALLOWED),
            }
        } else {
            response_with_code(StatusCode::NOT_FOUND)
        }
    };
    future::ok(response)
}

fn main() {
    let addr = ([127, 0, 0, 1], 9000).into();
    let builder = Server::bind(&addr);
    let user_db = Arc::new(Mutex::new(Slab::new()));
    let server = builder.serve(move || {
        let user_db = user_db.clone();
        service_fn(move |req| service_handler(req, &user_db))
    });

    let server = server.map_err(drop);
    hyper::rt::run(server);
}
```

```toml
[package]
name = "hyper_microservice"
version = "0.2.0"
edition = "2021"
publish = false

[dependencies]
hyper = "0.12"
futures = "0.1"
slab = "0.4"
regex = "0.1"
lazy_static = "0.1"
```

### **wrk** benchmark on release build `v0.2`

```sh
wrk -t1 -c10000 -d60s http://127.0.0.1:9000/

Running 1m test @ http://127.0.0.1:9000/
  1 threads and 10000 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     4.92ms    1.67ms  11.97ms   57.39%
    Req/Sec   108.59k     1.04k  112.20k    85.64%
  6482030 requests in 1.00m, 1.44GB read
  Socket errors: connect 8980, read 0, write 0, timeout 0
Requests/sec: 107847.42
Transfer/sec:     24.58MB
```

```sh
wrk -t10 -c10000 -d60s http://127.0.0.1:9000/

Running 1m test @ http://127.0.0.1:9000/
  10 threads and 10000 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.91ms    3.40ms  60.68ms   87.03%
    Req/Sec    34.69k     9.96k   75.05k    63.89%
  18661131 requests in 1.00m, 4.15GB read
  Socket errors: connect 8989, read 0, write 0, timeout 0
Requests/sec: 310531.78
Transfer/sec:     70.78MB
```

## source code v0.1

```rust
use futures::{future, Future};
use hyper::service::service_fn;
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use slab::Slab;
use std::fmt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

type UserId = u64;
struct UserData;
type UserDb = Arc<Mutex<Slab<UserData>>>;

const INDEX: &'static str = r#"
<!doctype html>
<html>
    <head>
        <title>hyper microservice</title>
    </head>
    <body>
        <h1>Microservices with Hyper</h1>
    </body>
</html>
"#;

fn main() {
    let addr = ([127, 0, 0, 1], 9000).into();
    let builder = Server::bind(&addr);
    let user_db = Arc::new(Mutex::new(Slab::new()));
    let server = builder.serve(move || {
        let user_db = user_db.clone();
        service_fn(move |req| service_handler(req, &user_db))
    });

    let server = server.map_err(drop);
    hyper::rt::run(server);
}

fn service_handler(
    req: Request<Body>,
    user_db: &UserDb,
) -> impl Future<Item = Response<Body>, Error = Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => future::ok(Response::new(INDEX.into())),
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();
            future::ok(response)
        }
    }
}
```

Dependencies :

```toml
[dependencies]
hyper = "0.12"
futures = "0.1"
slab = "0.4"
```

### **wrk** benchmark on debug build

```sh
wrk -t1 -c10000 -d60s http://127.0.0.1:9000/

Running 1m test @ http://127.0.0.1:9000/
  1 threads and 10000 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     9.40ms   15.35ms 712.65ms   98.57%
    Req/Sec    98.08k    17.45k  126.11k    47.40%
  5857534 requests in 1.00m, 1.30GB read
  Socket errors: connect 8984, read 0, write 0, timeout 0
Requests/sec:  97480.47
Transfer/sec:     22.22MB
```

```
wrk -t10 -c10000 -d60s http://127.0.0.1:9000/

Running 1m test @ http://127.0.0.1:9000/
  10 threads and 10000 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     9.84ms    5.75ms 329.24ms   85.55%
    Req/Sec    10.09k     5.91k   29.65k    74.12%
  6027926 requests in 1.00m, 1.34GB read
  Socket errors: connect 8993, read 0, write 0, timeout 0
Requests/sec: 100304.44
Transfer/sec:     22.86MB
```

### **wrk** benchmark on release build

```sh
wrk -t1 -c10000 -d60s http://127.0.0.1:9000/

Running 1m test @ http://127.0.0.1:9000/
  1 threads and 10000 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     4.83ms    1.50ms  12.65ms   58.69%
    Req/Sec   109.43k     1.27k  113.33k    80.24%
  6526490 requests in 1.00m, 1.45GB read
  Socket errors: connect 8980, read 0, write 0, timeout 0
Requests/sec: 108691.39
Transfer/sec:     24.77MB
```

```sh
wrk -t10 -c10000 -d60s http://127.0.0.1:9000/

Running 1m test @ http://127.0.0.1:9000/
  10 threads and 10000 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     3.13ms    6.91ms 448.93ms   89.85%
    Req/Sec    36.26k    13.58k  111.68k    68.39%
  19451742 requests in 1.00m, 4.33GB read
  Socket errors: connect 8993, read 0, write 0, timeout 0
Requests/sec: 323749.45
Transfer/sec:     73.79MB
```

### drill benchmark on release build

concurrency: 2
base: "http://localhost:9000"
iterations: 10000
rampup: 10

- Time taken for tests 6.4 seconds
- Total requests 10000
- Successful requests 10000
- Failed requests 0
- Requests per second 1557.70 [#/sec]
- Median time per request 0ms
- Average time per request 0ms
- Sample standard deviation 0ms
- 99.0'th percentile 0ms
- 99.5'th percentile 0ms
- 99.9'th percentile 1ms

---

concurrency: 10
base: "http://localhost:9000"
iterations: 1000000
rampup: 1000

- Time taken for tests 145.1 seconds
- Total requests 1000000
- Successful requests 1000000
- Failed requests 0
- Requests per second 6890.44 [#/sec]
- Median time per request 0ms
- Average time per request 0ms
- Sample standard deviation 0ms
- 99.0'th percentile 0ms
- 99.5'th percentile 1ms
- 99.9'th percentile 1ms

---

concurrency: 10
base: "http://localhost:9000"
iterations: 1000000
rampup: 1

- Time taken for tests 144.2 seconds
- Total requests 1000000
- Successful requests 1000000
- Failed requests 0
- Requests per second 6933.28 [#/sec]
- Median time per request 0ms
- Average time per request 0ms
- Sample standard deviation 0ms
- 99.0'th percentile 0ms
- 99.5'th percentile 1ms
- 99.9'th percentile 1ms

### drill benchmark on debug build

concurrency: 2
base: "http://localhost:9000"
iterations: 10000
rampup: 10

- Time taken for tests 7.6 seconds
- Total requests 10000
- Successful requests 10000
- Failed requests 0
- Requests per second 1322.51 [#/sec]
- Median time per request 0ms
- Average time per request 0ms
- Sample standard deviation 0ms
- 99.0'th percentile 1ms
- 99.5'th percentile 1ms
- 99.9'th percentile 1ms
