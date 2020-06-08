use std::net::TcpStream;
use std::io::{Write, BufRead, BufReader, Read};
use std::time::Duration;

mod ast;
mod color;
mod font;
mod style;

fn main() {
    println!("connecting");
    let mut stream = TcpStream::connect(include_str!("../server_address").trim()).unwrap();
    println!("connected");
    stream.write_all(b"hello\n").unwrap();
    //std::thread::sleep(Duration::from_secs(3));
    println!("sent request");
    stream.write_all(b"more\n").unwrap();
    //let mut response = String::new();
    //match BufReader::new(stream).read_line(&mut response) {
    //   Ok(size) => {
    //       println!("got response {}", response);
    //   }
    //   Err(e) => println!("error {}", e)
    //}
}
