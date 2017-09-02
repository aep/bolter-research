#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate tokio_core;

extern crate pretty_env_logger;

use std::io::{self, Write};

use self::futures::Future;
use self::futures::stream::Stream;

use self::hyper::Client;

pub fn main() {
    pretty_env_logger::init().unwrap();

    let url = "http://localhost:3333";
    let url = url.parse::<hyper::Uri>().unwrap();
    if url.scheme() != Some("http") {
        println!("This example only works with 'http' URLs.");
        return;
    }

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let client = Client::new(&handle);

    let work = client.get(url).and_then(|res| {
        println!("Response: {}", res.status());
        println!("Headers: \n{}", res.headers());

        res.body().for_each(|chunk| {
            io::stdout().write_all(&chunk).map_err(From::from)
        })
    }).map(|_| {
        println!("\n\nDone.");
    });

    core.run(work).unwrap();
}
