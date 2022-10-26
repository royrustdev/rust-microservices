# Hyper Server Benchmark

Benchmark with **wrk** tool

```
-c, --connections: total number of HTTP connections to keep open with each thread handling N = connections/threads

-d, --duration:    duration of the test, e.g. 2s, 2m, 2h

-t, --threads:     total number of threads to use
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
