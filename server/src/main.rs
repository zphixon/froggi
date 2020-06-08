use std::net::{TcpStream, TcpListener};
use std::io::{BufRead, Write, BufReader, Read};

fn handle_client(mut stream: TcpStream) {
    let mut request = [0; 512];
    match stream.read(&mut request) {
        Ok(size) => {
            println!("received request: {:?}", &request[0..size]);
            let size2 = stream.read(&mut request).unwrap();
            println!("more data? {:?}", &request[0..size2]);
            //if &request[0..size] == b"hello\n" {
            //    println!("sent response");
            //    stream.write_all("nerd\n".as_bytes()).unwrap();
            //}
        }
        Err(e) => {
            println!("error {}", e);
        }
    }
    // let mut request = String::new();
    // match BufReader::new(stream.try_clone().unwrap()).read_line(&mut request) {
    //    Ok(_) => {
    //        println!("received request: {:?}", request.as_bytes());
    //        if &request == "hello\n" {
    //            println!("sent response");
    //            stream.write_all("nerd\n".as_bytes()).unwrap();
    //        }
    //    }
    //    Err(e) => {
    //        println!("error {}", e);
    //    }
    // }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:11121").unwrap();
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
