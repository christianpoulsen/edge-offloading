use std::{thread};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write, Error};

use crate::helper;
use crate::server::{Server};


pub struct Controller {
    port: i32,
    num_of_servers: usize,
    addr: String,
    servers: Vec<Server>,
}

impl Controller {

    pub fn new() -> Controller {
        let port = 3333;
        let num_of_servers = 1;
        let mut controller = Controller {
            port,
            num_of_servers,
            addr: helper::socket_addr("0.0.0.0:", port),
            servers: start_servers(port, num_of_servers),
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

                    let task_size = read_task_size(&stream);

                    match task_size {
                        Ok(size) => {
                            println!("Message size: {}", size);
                            let server_addr = self.find_or_create_server(size).get_addr().clone();
                            thread::spawn(move || {      
                                connect_to_free_server(stream, server_addr);
                            });
                        },
                        Err(_) => {
                            println!("Couldn't read the size of the task")
                        }
                    }

                    // let free_addrs = Arc::clone(&free_server_addrs);
                    // // connection succeeded
                    // thread::spawn(move || {            
                    //     connect_to_free_server(stream, free_addrs);
                    // });
                }
                Err(e) => {
                    println!("Controller: Error: {}", e);
                    /* connection failed */
                }
            }
        }
        // close the socket server
        drop(listener);
    }

    fn find_or_create_server(&mut self, size: u64) -> &Server {

        let non_available = match self.find_server(&size) {
            Some(_) => false,
            None => true,
        };

        if non_available {
            let server = self.create_server();
            return server;
        } else {
            return self.find_server(&size).unwrap();
        }
             
    }

    fn find_server(&self, size: &u64) -> Option<&Server> {
        for server in self.servers.iter() {
            if server.get_size() <= size {
                return Some(server);
            }
        }
        return None;
    }

    fn create_server(&mut self) -> &Server {
        self.increment_port();
        let server_addr = helper::socket_addr("0.0.0.0", self.port);
        
        let server = Server::new(server_addr, 10);

        self.increment_num_of_servers();
        self.servers.push(server);

        return self.servers.last().unwrap();
    }

    fn increment_port(&mut self) {
        self.port += 1;
    }

    fn increment_num_of_servers(&mut self) {
        self.num_of_servers += 1;
    }

}

fn start_servers(port: i32, num: usize) -> Vec<Server> {

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

fn connect_to_free_server(client_stream: TcpStream, server_addr: String) {

    match TcpStream::connect(&server_addr) {
        Ok(server_stream) => {
            println!("Controller: Successfully connected to server at {}", server_addr);

            pass_msg_from_client_to_server(client_stream, server_stream);
        },
        Err(e) => {
            println!("Controller: Failed to connect: {}", e);
        }
    }
}

fn pass_msg_from_client_to_server(mut client_stream: TcpStream, mut server_stream: TcpStream) {
    // read the message from the client
    let mut client_data = [0 as u8; 50]; // using 50 byte buffer
    while match client_stream.read(&mut client_data) {
        Ok(size) => {
            let msg = &client_data[0..size];

            // write the message to the server
            server_stream.write(msg).unwrap();
            
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
    let mut server_data = [0 as u8; 6]; // using 6 byte buffer
    match server_stream.read_exact(&mut server_data) {
        Ok(_) => {
            // write the message from the server to the client
            client_stream.write(&server_data).unwrap();
            true
        },
        Err(e) => {
            println!("Controller: Failed to receive data: {}", e);
            false
        }
    }
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
