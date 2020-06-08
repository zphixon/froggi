use std::net::{TcpStream, TcpListener};
use std::io::{BufRead, Write, BufReader, Read};

fn handle_client(mut stream: TcpStream) {
    let mut request = Vec::new();
    match stream.read_to_end(&mut request) {
        Ok(size) => {
            println!("received request: {:?}", &request[0..size]);
            let size2 = stream.read_to_end(&mut request).unwrap();
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
