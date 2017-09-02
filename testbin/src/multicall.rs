use std::env::args;
mod client;
mod server;

fn main() {
    match args().nth(1) {
        Some(n) => {
            match &n as &str {
                "client" => client::main(),
                "server" => server::main(),
                &_ => println!("no such example")
            }
        },
        _ => {
            println!("specify main to run. valid mains:");
            println!("   server");
            println!("   client");
        }
    }
}
