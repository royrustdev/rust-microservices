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
