use std::{thread};
use std::net::{TcpListener, TcpStream, Shutdown, SocketAddrV4};
use std::io::{Read, Write, Error};
use std::str::from_utf8;

use crate::helper;
use crate::server::{Server};

pub struct Controller {
    port: i32,
    num_of_servers: i32,
    addr: String,
    servers: Vec<Server>,
}

static DEBUG: bool = true;
static BUSY_TIME: u64 = 20;

impl Controller {

    pub fn new() -> Controller {
        let init_port = 3333;
        let num_of_servers = 1;
        let port = init_port + num_of_servers;
        let mut controller = Controller {
            port,
            num_of_servers,
            addr: helper::socket_addr("0.0.0.0:", init_port),
            servers: start_servers(init_port, num_of_servers),
        };
        controller.run();
        return controller
    }

    pub fn run(&mut self) {
        println!("Starting controller...");
    
        // setup listener to connect to clients
        let listener = TcpListener::bind(self.addr.to_string()).unwrap();

        // accept connections and process them, spawning a new thread for each one
        if DEBUG { println!("Controller: Listening on {}", self.addr.to_string()) };
        for stream in listener.incoming() {

            match stream {
                Ok(mut stream) => {
                    if DEBUG { println!("Controller: New connection: {}", stream.peer_addr().unwrap()) };

                    let mut peek_buffer = [0; 64];
                    let total_size = stream.peek(&mut peek_buffer).unwrap();

                    if total_size > 8 {
                        match read_task_size(&stream) {
                            Ok(size) => {
                                let mut server_addr_to_update = "";
                                let mut buffer = [0; 56];
                                match stream.read(&mut buffer) {
                                    Ok(addr_size) => {
                                        let address = from_utf8(&buffer[0..addr_size]).unwrap();
                                        match address.parse::<SocketAddrV4>() {
                                            Ok(_) => {
                                                server_addr_to_update = address.clone();
                                            },
                                            Err(_) => {
                                                if DEBUG { println!("Couldn't parse incoming server address") };
                                            }
                                        }
                                        if !server_addr_to_update.is_empty() {
                                            self.update_server_size(String::from(server_addr_to_update), size, |x,y| x+y);
                                        }
                                    },
                                    Err(e) => {
                                        if DEBUG { println!("Couldn't read the rest: {}", e) };
                                    }
                                }
                            },
                            Err(_) => {}
                        }
                    } else {
                        match read_task_size(&stream) {
                            Ok(size) => {
                                let server_index = self.find_or_create_server(size);
                                let server_addr_to_connect = self.get_server_addr(server_index);
                                self.update_server_size(server_addr_to_connect.clone(), size, |x,y| x-y);

                                thread::spawn(move || {
                                    stream.write(server_addr_to_connect.as_bytes()).unwrap();
                                });
                            },
                            Err(_) => if DEBUG { println!("Couldn't read the size of the task") },
                        }
                    }
                }
                Err(e) => if DEBUG { println!("Controller: Error: {}", e) },
            }
        }
        // close the socket server
        drop(listener);
    }

    fn find_or_create_server(&mut self, size: u64) -> usize {
        if DEBUG { println!("size {}", size) };
        return match self.find_server(size) {
            Some(index) => index,
            None => self.create_server(),
        };             
    }

    fn find_server(&self, size: u64) -> Option<usize> {
        let length = self.servers.len();
        for i in 0..length {
            if DEBUG { println!("server: {}; size: {}", self.servers[i].get_addr(), self.servers[i].get_size()) };
            if self.servers[i].get_size() >= size {
                if DEBUG { println!("index {}", i) };
                return Some(i);
            }
        }
        
        return None;
    }

    fn create_server(&mut self) -> usize {
        if DEBUG { println!("Creating new server") };
        self.increment_port();
        let server_addr: String = helper::socket_addr("0.0.0.0:", self.port);

        let server = Server::new(server_addr, BUSY_TIME);

        self.increment_num_of_servers();
        self.servers.push(server);

        return self.servers.len() - 1;
    }

    fn get_server_addr(&self, index: usize) -> String {
        return String::from(self.servers[index].get_addr());
    }

    fn update_server_size(&mut self, addr: String, size: u64, update: fn(u64, u64) -> u64) {
        let mut index = 0;
        for s in self.servers.iter() {
            if *s.get_addr() == addr {
                break
            }
            index += 1;
        }
        let mut server = self.servers[index].clone();
        let current_size = &server.get_size();
        let new_size = update(*current_size, size);
        server.set_size(new_size);
        self.servers[index] = server;
    }

    fn increment_port(&mut self) {
        self.port += 1;
    }

    fn increment_num_of_servers(&mut self) {
        self.num_of_servers += 1;
    }

}

fn start_servers<'a>(port: i32, num: i32) -> Vec<Server> {

    if DEBUG { println!("Controller: Starting the {} servers...", num) };
    
    let mut servers: Vec<Server> = Vec::new();

    let mut port = port;

    for _i in 0..num {
        port += 1;

        let server_addr = helper::socket_addr("0.0.0.0:", port);

        let server = Server::new(server_addr, BUSY_TIME);
            
        servers.push(server);
    }

    return servers;
}

fn read_task_size(mut client_stream: &TcpStream) -> Result<u64, Error> {
    let mut msg_size_buffer: [u8; 8] = [0; 8];
    return match client_stream.read_exact(&mut msg_size_buffer) {
        Ok(_) => {
            let size = u64::from_be_bytes(msg_size_buffer);
            Ok(size)
        },
        Err(e) => {
            if DEBUG { println!("Controller: An error occurred, terminating connection with {}", client_stream.peer_addr().unwrap()) };
            client_stream.shutdown(Shutdown::Both).unwrap();
            panic!("Problem reading the task size: {}", e);
        }
    }
}
