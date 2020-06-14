use std::io::{BufRead, BufReader, Cursor, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn main() {
    froggi::hello();
    println!("connecting");
    let mut stream = TcpStream::connect(include_str!("../server_address").trim()).unwrap();
    println!("connected");
    // std::thread::sleep(Duration::from_secs(1));
    println!("sending hello");
    stream.write_all(b"hello\n").unwrap();
    stream.write_all(&[]).unwrap();
    println!("sent");
    std::thread::sleep(Duration::from_secs(5));
    println!("end");
}
