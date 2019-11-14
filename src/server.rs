use std::{thread, time};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};

pub fn sleep(sec: u64) {
    let dur = time::Duration::from_secs(sec);

    println!("Sleep for {} seconds", dur.as_secs());
    thread::sleep(dur);
}

pub fn busy_loop(sec: u64) {
    let dur = time::Duration::from_secs(sec);

    let now = time::Instant::now();
    let mut earlier = now.elapsed();

    println!("Busy for {} seconds", sec);

    while now.elapsed() < dur {
        if earlier.as_secs() < now.elapsed().as_secs() {
            if now.elapsed().as_secs() != (sec-1) {
                println!("Busy for {} more seconds ...", (sec-1)-earlier.as_secs());
            } else if now.elapsed().as_secs() == (sec-1)  {
                println!("Busy for {} more second ...", (sec-1)-earlier.as_secs());
            }
            earlier = now.elapsed();
        }
    }
}


pub fn handle_client(mut stream: TcpStream, dur_secs: u64) {
    busy_loop(dur_secs);
    sleep(dur_secs);
    busy_loop(dur_secs);
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            // echo everything!
            stream.write(&data[0..size]).unwrap();
            if size == 0 { false } else { true }
    },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } { }
}

pub fn run(dur_secs: u64) {
    println!("Starting server...");
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let address = stream.peer_addr().unwrap();
                println!("New connection: {}", stream.peer_addr().unwrap());
                let handle = thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream, dur_secs)
                });

                handle.join().unwrap();
                println!("End connection with: {}", address);
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}
