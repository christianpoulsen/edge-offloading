use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

pub struct Client<'a> {
    connect_ipv4: &'a str,
    connect_port: &'a str,
    buffer: Vec<u8>,
    msg: &'a [u8],
}

impl<'client> Client<'client> {

    pub fn new<'a>() -> Client<'a> {

        let msg = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas laoreet hendrerit tempor. Donec ut tellus velit. Mauris egestas ac risus in volutpat. Quisque in purus id nisi tincidunt vehicula. Maecenas non nisi vitae risus congue rutrum ut et leo. Aliquam tincidunt, nunc sit amet aliquet gravida, elit elit sagittis risus, molestie porta lorem odio a sapien. Cras nec sollicitudin turpis, quis lacinia sapien. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Sed consequat id ipsum non aliquet. Proin eu lacus faucibus, elementum lorem et, mattis justo. Aliquam eu nisl velit.!";

        let mut size_buffer: [u8; 8] = [0; 8];
        
        let size = msg.len().to_be_bytes();
        
        let mut counter = 0;
        for byte in size.iter() {
            size_buffer[counter] = *byte;
            counter += 1;
        }

        let mut buffer = Vec::new();
        buffer.extend_from_slice(&size_buffer);
        buffer.extend_from_slice(msg);

        let mut client = Client {
            connect_ipv4: "0.0.0.0",
            connect_port: "3333",
            buffer,
            msg,
        };
        client.run();
        return client
    }

    fn run(&mut self) {
        println!("Starting client...");
        let connecting_addr = format!("{}:{}", self.connect_ipv4, self.connect_port);
        match TcpStream::connect(connecting_addr) {
            Ok(mut stream) => {
                println!("Successfully connected to server in port 3333");

                stream.write_all(self.buffer.as_mut()).unwrap();
                println!("Sent message, awaiting reply...");

                let mut data: Vec<u8> = Vec::new();
                match stream.read_to_end(&mut data) {
                    Ok(_) => {
                        let test = data.as_slice();
                        let equal = test.eq_ignore_ascii_case(self.msg);
                        if equal {
                            println!("Reply is ok!");
                        } else {
                            let text = from_utf8(&data).unwrap();
                            println!("Unexpected reply: {}", text);
                        }
                    },
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
            },
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
        println!("Terminated.");
    }
}