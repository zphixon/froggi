use std::io::{BufRead, BufReader, Cursor, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn main() {
    froggi::hello();
    println!("connecting");
    let mut stream = TcpStream::connect(include_str!("../server_address").trim()).unwrap();
    let request = froggi::request::Request::new("index.fml").unwrap();
    stream.write_all(&request.into_bytes()).unwrap();
    println!("sent");
}
