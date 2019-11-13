use std::env;

mod client;
mod server;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args[1] == "server" {
        server::run();
    } else if args[1] == "client" {
        client::run();
    } else {
        println!("error");
    }
}
