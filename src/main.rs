use std::env;

mod client;
mod server;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args[1] == "server" {
        let mut seconds = 5;
        let arg2 = args.get(2);
        if arg2 != None {
            seconds = match args[2].parse::<u64>() {
                Ok(i) => i,
                Err(_e) => {
                    seconds
                }
            };
        }
        server::run(seconds);
    } else if args[1] == "client" {
        client::run();
    } else {
        println!("error");
    }
}
