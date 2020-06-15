use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    let mut request = [0u8; 3];
    stream.read(&mut request).expect("froggi header");

    let version = request[0];
    let path_len = froggi::deserialize_bytes(request[1], request[2]);

    let mut path_buf = Vec::with_capacity(path_len);
    stream.read_exact(&mut path_buf).unwrap();

    let path = String::from_utf8(path_buf).unwrap();

    println!("request (version {}, length {}): {}", version, path_len, path);
}

fn main() {
    froggi::hello();
    let listener = TcpListener::bind("0.0.0.0:11121").unwrap();
    println!("listening");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new client");
                std::thread::spawn(move || handle_client(stream));
            }
            Err(e) => {
                println!("error {}", e);
            }
        }
    }
}

