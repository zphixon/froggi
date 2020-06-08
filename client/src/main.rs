use std::net::TcpStream;
use std::io::{Write, BufRead, BufReader, Read, Cursor};
use std::time::Duration;

mod ast;
mod color;
mod font;
mod style;

fn main() {
    println!("connecting");
    let mut stream = TcpStream::connect(include_str!("../server_address").trim()).unwrap();
    println!("connected");
    // std::thread::sleep(Duration::from_secs(1));
    println!("sending hello");
    stream.write_all(b"hello\n").unwrap();
    println!("sent hello");
    // std::thread::sleep(Duration::from_secs(1));
    println!("sending more");
    stream.write_all(b"more\n").unwrap();
    println!("sent more");
    // std::thread::sleep(Duration::from_secs(1));
    println!("sending nul");
    stream.write_all(b"\0").unwrap();
    println!("sent nul");
    stream.write_all(b"").unwrap();
    stream.flush();
    // std::thread::sleep(Duration::from_secs(1));
    println!("sent request");
    //let mut response = String::new();
    //match BufReader::new(stream).read_line(&mut response) {
    //   Ok(size) => {
    //       println!("got response {}", response);
    //   }
    //   Err(e) => println!("error {}", e)
    //}
}
