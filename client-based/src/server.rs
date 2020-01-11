use std::{thread, time};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
// use std::str::from_utf8;

#[derive(Clone)]
pub struct Server {
    addr: String,
    dur_secs: u64,
    size: u64,
}

impl Server {

    pub fn new(addr: String, dur_secs: u64) -> Server {
        let server: Server = Server {
            addr,
            dur_secs,
            size: 1000,
        };
        let running_server = server.clone();

        thread::spawn(move || {
            running_server.run();
        });
        
        return server;
    }

    fn run(&self) {
        let listener = TcpListener::bind(self.addr.to_string()).unwrap();
        // accept connections and process them, spawning a new thread for each one
        println!("Listening on {}", self.addr.to_string());
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Server [{}]: New connection: {}", self.addr, stream.peer_addr().unwrap());
                    let sec = self.dur_secs.clone();
                    thread::spawn(move || {
                        // connection succeeded
                        handle_client(stream, sec)
                    });

                }
                Err(e) => {
                    println!("Server [{}]. Error: {}", self.addr, e);
                    /* connection failed */
                }
            }
        }
        // close the socket server
        drop(listener);
    }

    pub fn get_size(&self) -> u64 {
        return self.size;
    }

    pub fn set_size(&mut self, size: u64) {
        self.size = size;
    }

    pub fn get_addr(&self) -> &String {
        return &self.addr;
    }

}


fn busy_loop(sec: u64) {
    let dur = time::Duration::from_secs(sec);

    let now = time::Instant::now();
    let mut earlier = now.elapsed();

    // println!("Server: Busy for {} seconds", sec);

    while now.elapsed() < dur {
        if earlier.as_secs() < now.elapsed().as_secs() {
            if now.elapsed().as_secs() != (sec-1) {
                // println!("Server: Busy for {} more seconds ...", (sec-1)-earlier.as_secs());
            } else if now.elapsed().as_secs() == (sec-1)  {
                // println!("Server: Busy for {} more second ...", (sec-1)-earlier.as_secs());
            }
            earlier = now.elapsed();
        }
    }
}


fn handle_client(mut stream: TcpStream, dur_sec: u64) {
    busy_loop(dur_sec);
    let mut data = [0; 512];
    while match stream.read(&mut data) {
        Ok(size) => {
            // echo everything!
            // println!("2) Server msg: {:?}\n", from_utf8(&data[0..size]).unwrap());
            stream.write(&data[0..size]).unwrap();
            if size == 0 { false } else { true }
    },
        Err(_) => {
            println!("Server: An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } { }
}