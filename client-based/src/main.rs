use std::env;

mod helper;

mod server;
mod controller;
mod client;

use crate::server::{Server};
use crate::controller::{Controller};
use crate::client::{Client};

extern crate rand;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args[1] == "controller" {
        Controller::new();
    } else if args[1] == "client" {
        Client::new();
    } else if args[1] == "server" {
        Server::new("0.0.0.0:3333".to_owned(), 10);
    } else {
        println!("error");
    }
}
