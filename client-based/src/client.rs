use std::net::{TcpStream};
use std::{thread, time};
use std::io::{Read, Write};
// use std::str::from_utf8;
use rand::{thread_rng, Rng};

pub struct Client<'a> {
    connect_ipv4: &'a str,
    connect_port: &'a str,
    // buffer: Vec<u8>,
    // msg: &'a [u8],
}

impl<'client> Client<'client> {

    pub fn new<'a>() -> Client<'a> {

        let lorem_string = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas laoreet hendrerit tempor. Donec ut tellus velit. Mauris egestas ac risus in volutpat. Quisque in purus id nisi tincidunt vehicula. Maecenas non nisi vitae risus congue rutrum ut et leo. Aliquam tincidunt, nunc sit amet aliquet gravida, elit elit sagittis risus, molestie porta lorem odio a sapien. Cras nec sollicitudin turpis, quis lacinia sapien. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Sed consequat id ipsum non aliquet. Proin eu lacus faucibus, elementum lorem et, mattis justo. Aliquam eu nisl velit!";
        let mut size_buffer: [u8; 8] = [0; 8];
        
        let mut rng = thread_rng();
        let size = rng.gen_range(50, lorem_string.len());
        let msg: &[u8] = &lorem_string[0..size];
        
        let mut counter = 0;
        for byte in size.to_be_bytes().iter() {
            size_buffer[counter] = *byte;
            counter += 1;
        }

        let mut buffer = Vec::new();
        buffer.extend_from_slice(&size_buffer);
        buffer.extend_from_slice(msg);

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
        println!("Starting client...");

        let lorem_string = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas laoreet hendrerit tempor. Donec ut tellus velit. Mauris egestas ac risus in volutpat. Quisque in purus id nisi tincidunt vehicula. Maecenas non nisi vitae risus congue rutrum ut et leo. Aliquam tincidunt, nunc sit amet aliquet gravida, elit elit sagittis risus, molestie porta lorem odio a sapien. Cras nec sollicitudin turpis, quis lacinia sapien. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Sed consequat id ipsum non aliquet. Proin eu lacus faucibus, elementum lorem et, mattis justo. Aliquam eu nisl velit!";
        let mut size_buffer: [u8; 8] = [0; 8];

        let connecting_addr = format!("{}:{}", self.connect_ipv4, self.connect_port);
        
        loop {
            match TcpStream::connect(&connecting_addr) {
                Ok(stream) => {
                    let id = rand::thread_rng().gen_ascii_chars().take(10).collect::<String>();
                    print!("{} : Writing a new task : ", id);

                    let mut rng = thread_rng();
                    let size = rng.gen_range(50, lorem_string.len());
                    let msg: &[u8] = &lorem_string[0..size];
                    
                    let mut counter = 0;
                    for byte in size.to_be_bytes().iter() {
                        size_buffer[counter] = *byte;
                        counter += 1;
                    }

                    let mut buffer: Vec<u8> = Vec::new();
                    buffer.extend_from_slice(&size_buffer);
                    buffer.extend_from_slice(msg);

                    print!("Message size is {} : ", size);

                    match stream.try_clone() {
                        Ok(mut cloned_stream) => {
                            thread::spawn(move || {
                                println!("Spawning thread and writing data;");
                                cloned_stream.write_all(buffer.as_mut()).unwrap();

                                let mut incoming_msg: Vec<u8> = Vec::new();
                                let mut data = [0; 512];
                                while match cloned_stream.read(&mut data) {
                                    Ok(size) => {
                                        incoming_msg.extend_from_slice(&data[0..size]);
                                        if size < 512 { false } else { true }
                                    },
                                    Err(e) => {
                                        println!("Failed to receive data: {}", e);
                                        false
                                    }
                                } { }
                                if incoming_msg.as_slice().eq_ignore_ascii_case(&msg) {
                                    println!("{} : Reply is ok!", id);
                                    // println!("{}\n", from_utf8(&incoming_msg).unwrap());
                                } else {
                                    println!("{} : Unexpected reply!", id);
                                }
                            });
                        },
                        Err(_) => println!("{}: Failed to clone stream", id),
                    }
                },
                Err(e) => {
                    println!("Failed to connect: {}", e);
                }
            }
            let sleep_time = time::Duration::from_secs(3);
            thread::sleep(sleep_time);
        }
    }
}