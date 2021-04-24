use froggi::request::Request;
use froggi::response::{Response, ResponseKind};

use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    let request = Request::from_bytes(&mut stream).unwrap();

    println!("request: {:?}", request);

    unsafe {
        match PAGES.as_ref().unwrap().get(request.request()) {
            Some(page) => stream.write_all(&page.bytes()).unwrap(),
            None => stream
                .write_all(&NOT_FOUND.as_ref().unwrap().bytes())
                .unwrap(),
        }
    }
}

static mut NOT_FOUND: Option<Response> = None;
static mut PAGES: Option<HashMap<String, Response>> = None;

fn main() {
    let mut pages = HashMap::new();

    println!("reading pages");
    for item in std::fs::read_dir("pages").unwrap() {
        let item = item.unwrap();
        if item.metadata().unwrap().is_file()
            && item.file_name().to_str().unwrap().ends_with(".fml")
        {
            println!("{}", item.file_name().to_str().unwrap());
            pages.insert(
                item.file_name().into_string().unwrap(),
                froggi::response_from_file(item.path()).unwrap(),
            );
        }
    }

    unsafe {
        PAGES = Some(pages);
        NOT_FOUND = Some(
            Response::new(ResponseKind::Error, String::from("('not found')"), vec![]).unwrap(),
        );
    }

    let listener = TcpListener::bind("0.0.0.0:11121").unwrap();
    println!(
        "listening at {}. run this binary from froggi-server dir!",
        listener.local_addr().unwrap()
    );

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
