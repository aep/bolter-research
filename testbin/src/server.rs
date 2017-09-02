#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;

use self::futures::future::FutureResult;

use self::hyper::{Get, Post, StatusCode};
use self::hyper::header::ContentLength;
use self::hyper::server::{Http, Service, Request, Response};

static INDEX: &'static [u8] = b"Try POST /echo";

struct Echo;

impl Service for Echo {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        futures::future::ok(match (req.method(), req.path()) {
            (&Get, "/") | (&Get, "/echo") => {
                Response::new()
                    .with_header(ContentLength(INDEX.len() as u64))
                    .with_body(INDEX)
            },
            (&Post, "/echo") => {
                let mut res = Response::new();
                if let Some(len) = req.headers().get::<ContentLength>() {
                    res.headers_mut().set(len.clone());
                }
                res.with_body(req.body())
            },
            _ => {
                Response::new()
                    .with_status(StatusCode::NotFound)
            }
        })
    }

}


pub fn main() {
    pretty_env_logger::init().unwrap();
    let addr = "127.0.0.1:3333".parse().unwrap();

    let server = Http::new().bind(&addr, || Ok(Echo)).unwrap();
    println!("Listening on http://{} with 1 thread.", server.local_addr().unwrap());
    server.run().unwrap();
}
