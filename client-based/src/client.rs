use std::net::{TcpStream,SocketAddrV4};
use std::{thread, time};
use std::io::{Read, Write};
use std::str::from_utf8;
use rand::{Rng};
// use std::fs;
// use std::sync::{Arc, Mutex};
// use std::convert::TryInto;
use ::time::{Instant};

pub struct Client<'a> {
    connect_ipv4: &'a str,
    connect_port: &'a str,
    // buffer: Vec<u8>,
    // msg: &'a [u8],
}

static DEBUG: bool = false;
static LOG: bool = true;

static SLEEP_TIME: u64 = 2;
static TASK_SIZE: usize = 10; // TASK_SIZE * 10

impl<'client> Client<'client> {

    pub fn new<'a>() -> Client<'a> {

        // let lorem_string = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas laoreet hendrerit tempor. Donec ut tellus velit. Mauris egestas ac risus in volutpat. Quisque in purus id nisi tincidunt vehicula. Maecenas non nisi vitae risus congue rutrum ut et leo. Aliquam tincidunt, nunc sit amet aliquet gravida, elit elit sagittis risus, molestie porta lorem odio a sapien. Cras nec sollicitudin turpis, quis lacinia sapien. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Sed consequat id ipsum non aliquet. Proin eu lacus faucibus, elementum lorem et, mattis justo. Aliquam eu nisl velit!";
        // let mut size_buffer: [u8; 8] = [0; 8];
        
        // let mut rng = thread_rng();
        // let size = 100; //rng.gen_range(50, lorem_string.len());
        // let msg: &[u8] = &lorem_string[0..size];
        
        // let mut counter = 0;
        // for byte in size.to_be_bytes().iter() {
        //     size_buffer[counter] = *byte;
        //     counter += 1;
        // }

        // let mut buffer = Vec::new();
        // buffer.extend_from_slice(&size_buffer);
        // buffer.extend_from_slice(msg);

        let mut client = Client {
            connect_ipv4: "0.0.0.0",
            connect_port: "3333",
            // buffer,
            // msg,
        };
        client.run();
        return client
    }

    fn run(&mut self) {
        if DEBUG { println!("Starting client...") };
        if LOG { println!("latency") };

        let lorem_string = "0123456789"; 
        //b"LoremIpsum"; // dolor sit amet, consectetur adipiscing elit. Maecenas laoreet hendrerit tempor. Donec ut tellus velit. Mauris egestas ac risus in volutpat. Quisque in purus id nisi tincidunt vehicula. Maecenas non nisi vitae risus congue rutrum ut et leo. Aliquam tincidunt, nunc sit amet aliquet gravida, elit elit sagittis risus, molestie porta lorem odio a sapien. Cras nec sollicitudin turpis, quis lacinia sapien. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Sed consequat id ipsum non aliquet. Proin eu lacus faucibus, elementum lorem et, mattis justo. Aliquam eu nisl velit!";

        // let size_data: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
        // let time_data: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));

        // let mut rerepeat = 0;

        let timer = Instant::now();
        
        while timer.elapsed().whole_minutes() < 10 {

            // let mut rng = thread_rng();
            // let size = rng.gen_range(50, lorem_string.len());
            // let msg = lorem_string.repeat(10).as_bytes();
            // let msg: &[u8] = lorem_string.repeat(10).as_bytes();//&lorem_string[0..size];
            
            let mut size_buffer: [u8; 8] = [0; 8];
            
            let controller_addr = format!("{}:{}", self.connect_ipv4, self.connect_port);
            let id = rand::thread_rng().gen_ascii_chars().take(10).collect::<String>();

            thread::spawn(move || {

                let start = timer.elapsed().whole_microseconds();

                let mut buffer1 = [0; 512];
                let mut server_address: &str = "";


                let temp = lorem_string.clone().repeat(TASK_SIZE);
                let msg = temp.as_bytes();

                let size = msg.len();

                // println!("repeat: {}; size: {}; msg: {:?}", repeat, size, msg);

                let mut counter = 0;
                for byte in size.to_be_bytes().iter() {
                    size_buffer[counter] = *byte;
                    counter += 1;
                }
            
                if DEBUG { println!("Connecting to controller at: {}", controller_addr) };
                match TcpStream::connect(&controller_addr) {
                    Ok(mut controller_stream) => {
                        controller_stream.write(&size_buffer).unwrap();
                        match controller_stream.read(&mut buffer1) {
                            Ok(size) => {
                                server_address = from_utf8(&buffer1[0..size]).unwrap();
                            },
                            Err(e) => {
                                if DEBUG { println!("Failed to receive data: {}", e) };
                            }
                        }
                    },
                    Err(e) => {
                        if DEBUG { println!("Failed to connect to controller: {}", e) };
                    }
                }

                let mut buffer2 = [0; 512];

                match server_address.parse::<SocketAddrV4>() {
                    Ok(address) => {
                        

                        if DEBUG { println!("Connecting to server at: {}", address) };
                        match TcpStream::connect(address) {
                            Ok(mut server_stream) => {


                                server_stream.write_all(&msg).unwrap();

                                let mut incoming_msg: Vec<u8> = Vec::new();
                                while match server_stream.read(&mut buffer2) {
                                    Ok(size) => {
                                        incoming_msg.extend_from_slice(&buffer2[0..size]);
                                        if size < 512 { false } else { true }
                                    },
                                    Err(e) => {
                                        if DEBUG { println!("Failed to receive data: {}", e) };
                                        false
                                    }
                                } { }
                                if incoming_msg.as_slice().eq_ignore_ascii_case(&msg) {
                                    if DEBUG { println!("{} : Reply is ok!", id) };
                                    // println!("{}\n", from_utf8(&incoming_msg).unwrap());
                                } else {
                                    if DEBUG { println!("{} : Unexpected reply!", id) };
                                }
                            },
                            Err(e) => {
                                if DEBUG { println!("Failed to connect to server: {}", e) };
                            }
                        }
                    },
                    Err(_) => if DEBUG { println!("Failed to parse ipv4 address: {}", server_address) },
                }

                let mut server_update = Vec::new();
                server_update.extend_from_slice(&size_buffer);
                server_update.extend_from_slice(server_address.as_bytes());

                // println!("Connecting to controller again at: {}", controller_addr);
                match TcpStream::connect(&controller_addr) {
                    Ok(mut controller_stream) => {
                            controller_stream.write(&server_update.as_slice()).unwrap();
                    },
                    Err(e) => {
                        if DEBUG { println!("Failed to connect to controller: {}", e) };
                    }
                }

                let end = timer.elapsed().whole_microseconds();
                let latency = end - start;

                if LOG { println!("{}", latency ) };

            });

            let sleep_time = time::Duration::from_secs(SLEEP_TIME);
            thread::sleep(sleep_time);
        }
    }
}