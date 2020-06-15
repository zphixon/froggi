use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    let request = froggi::request::Request::from_bytes(&mut stream).unwrap();

    println!(
        "request (version {}, length {}): {}",
        request.version(),
        request.path().len(),
        request.path()
    );
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
