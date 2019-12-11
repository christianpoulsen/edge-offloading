use std::{thread};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write, Error};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::str::from_utf8;

use crate::helper;
use crate::server::{Server};

#[derive(Clone)]
struct Connection {
    running: Arc<AtomicBool>,
    server_index: usize,
    server_addr: String,
    task_size: u64,
}

impl Connection {
    pub fn get_running(&self) -> &Arc<AtomicBool> {
        return &self.running
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.server_addr == other.server_addr
    }
}

pub struct Controller {
    port: i32,
    num_of_servers: i32,
    addr: String,
    servers: Vec<Server>,
    connections: Vec<Connection>,
}

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
            connections: Vec::new(),
        };
        controller.run();
        return controller
    }

    pub fn run(&mut self) {
        println!("Starting controller...");
    
        // setup listener to connect to clients
        let listener = TcpListener::bind(self.addr.to_string()).unwrap();

        // accept connections and process them, spawning a new thread for each one
        println!("Controller: Listening on {}", self.addr.to_string());
        for stream in listener.incoming() {

            match stream {
                Ok(stream) => {
                    println!("Controller: New connection: {}", stream.peer_addr().unwrap());

                    if self.connections.len() > 0 {
                        let mut connections_to_remove: Vec<Connection> = Vec::new();
                        for i in 0..self.connections.len() {
                            let running = Arc::clone(self.connections[i].get_running());

                            if !running.load(Ordering::Relaxed) {
                                println!("Updating server size and removing connection");
                                self.update_server_size(self.connections[i].server_index, self.connections[i].task_size, |x,y| x+y);
                                connections_to_remove.push(self.connections[i].clone());
                            }
                        }
                        self.connections.retain(|con| !connections_to_remove.contains(&con));
                    }

                    let task_size = read_task_size(&stream);

                    match task_size {
                        Ok(size) => {
                            println!("Message size: {}", size);
                            let server_index = self.find_or_create_server(size);
                            println!("Server index: {}", server_index);
                            let server_addr = self.get_server_addr(server_index);
                            println!("Server addr: {}", server_addr);
                            self.update_server_size(server_index, size, |x,y| x-y);
                            let new_connection = Connection {
                                running: Arc::new(AtomicBool::new(true)),
                                task_size: size,
                                server_addr: server_addr.clone(),
                                server_index,
                            };
                            let running = Arc::clone(new_connection.get_running());
                            self.connections.push(new_connection);
                            println!("Spawning thread that handles connection with address: {}", server_addr);
                            thread::spawn(move || {
                                connect_to_free_server(stream, server_addr.as_str(), running);
                            });                            
                        },
                        Err(_) => println!("Couldn't read the size of the task"),
                    }
                }
                Err(e) => println!("Controller: Error: {}", e),
            }
        }
        // close the socket server
        drop(listener);
    }

    fn find_or_create_server(&mut self, size: u64) -> usize {
        return match self.find_server(size) {
            Some(index) => index,
            None => self.create_server(),
        };             
    }

    fn find_server(&self, size: u64) -> Option<usize> {
        println!("Finding server...");
        let length = self.servers.len();
        for i in 0..length {
            if self.servers[i].get_size() >= size {
                return Some(i);
            }
        }
        
        return None;
    }

    fn create_server(&mut self) -> usize {
        println!("Creating new server");
        self.increment_port();
        let server_addr: String = helper::socket_addr("0.0.0.0:", self.port);
        
        println!("A, {}", server_addr);
        let server = Server::new(server_addr, 10);

        println!("B");
        self.increment_num_of_servers();
        self.servers.push(server);

        println!("C");
        return self.servers.len() - 1;
    }

    fn get_server_addr(&self, index: usize) -> String {
        return String::from(self.servers[index].get_addr());
    }

    fn update_server_size(&mut self, index: usize, size: u64, update: fn(u64, u64) -> u64) {
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

    println!("Controller: Starting the {} servers...", num);
    
    let sec = 10;
    let mut servers: Vec<Server> = Vec::new();

    let mut port = port;

    for _i in 0..num {
        port += 1;

        let server_addr = helper::socket_addr("0.0.0.0:", port);

        let server = Server::new(server_addr, sec);
            
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
            println!("Controller: An error occurred, terminating connection with {}", client_stream.peer_addr().unwrap());
            client_stream.shutdown(Shutdown::Both).unwrap();
            panic!("Problem reading the task size: {}", e);
        }
    }
}

fn connect_to_free_server(client_stream: TcpStream, server_addr: &str, running: Arc<AtomicBool>) {

    match TcpStream::connect(&server_addr) {
        Ok(server_stream) => {
            println!("Controller: Successfully connected to server at {}", server_addr);

            pass_msg_from_client_to_server(client_stream, server_stream);
        },
        Err(e) => {
            println!("Controller: Failed to connect: {}", e);
        }
    }

    println!("Setting running to false");
    running.store(false, Ordering::Relaxed);
}

fn pass_msg_from_client_to_server(mut client_stream: TcpStream, mut server_stream: TcpStream) {
    // read the message from the client
    let mut client_data = [8; 512];
    while match client_stream.read(&mut client_data) {
        Ok(size) => {
            let msg = &client_data[0..size];

            // println!("1) pass_msg_from_client_to_server: {:?}\n", from_utf8(msg).unwrap());

            // write the message to the server
            server_stream.write_all(msg).unwrap();
            
            // continuing passing data as long, as there is more data to read
            if size != 0 { 
                pass_msg_from_server_to_client(&server_stream, &client_stream)
            } else { 
                false 
            }
        },
        Err(_) => {
            println!("Controller: An error occurred, terminating connection with {}", client_stream.peer_addr().unwrap());
            client_stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } { }
}

fn pass_msg_from_server_to_client(mut server_stream: &TcpStream, mut client_stream: &TcpStream) -> bool {
    // read the message from the server
    let mut server_data = [0; 512];
    match server_stream.read(&mut server_data) {
        Ok(size) => {
            // println!("3) pass_msg_from_server_to_client: {:?}\n", from_utf8(&server_data[0..size]).unwrap());
            // write the message from the server to the client
            client_stream.write(&server_data[0..size]).unwrap();
            true
        },
        Err(e) => {
            println!("Controller: Failed to receive data: {}", e);
            false
        }
    }
}
