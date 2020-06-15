use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    let mut request = [0u8; 3];
    stream.read(&mut request).expect("froggi header");

    let version = request[0];
    let path_len = froggi::deserialize_bytes(request[1], request[2]);

    let mut total_read = 0;
    let mut path_buf = Vec::with_capacity(path_len);
    let mut buffer = [0; 64];
    while let Ok(n) = stream.read(&mut buffer) {
        total_read += n;
        path_buf.extend_from_slice(&buffer);
    }

    let path = String::from_utf8(path_buf).unwrap();

    println!("request (version {}): {}", version, path);
    //let mut request = Vec::new();
    //match stream.read_to_end(&mut request) {
    //    Ok(size) => {
    //        println!("received request: {:?}", &request[0..size]);
    //        let size2 = stream.read_to_end(&mut request).unwrap();
    //        println!("more data? {:?}", &request[0..size2]);
    //        //if &request[0..size] == b"hello\n" {
    //        //    println!("sent response");
    //        //    stream.write_all("nerd\n".as_bytes()).unwrap();
    //        //}
    //    }
    //    Err(e) => {
    //        println!("error {}", e);
    //    }
    //}
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

