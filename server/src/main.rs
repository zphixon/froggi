use froggi::request::Request;
use froggi::response::{Response, ResponseKind};

use froggi::{markup, response, ErrorKind, FroggiError};
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};

fn handle_client(mut stream: TcpStream, page_store: Arc<Mutex<PageStore>>) {
    let request = Request::from_bytes(&mut stream).unwrap();

    println!("request: {:?}", request);

    let mut page_store = page_store.lock().unwrap();
    match page_store.page(request.request()) {
        Some(page) => stream.write_all(page).unwrap(),
        None => stream.write_all(&page_store.not_found).unwrap(),
    }
}

// TODO convert to hash<string, vec<u8>> fully cause this is stupid
struct PageStore {
    pages: HashMap<String, Response>,
    page_cache: HashMap<String, Vec<u8>>,
    not_found: Vec<u8>,
}

impl PageStore {
    fn new() -> PageStore {
        PageStore {
            pages: HashMap::new(),
            page_cache: HashMap::new(),
            not_found: Response::new(ResponseKind::Error, String::from("('not found')"), vec![])
                .unwrap()
                .bytes(),
        }
    }

    fn add_page(&mut self, name: String, response: Response) {
        self.pages.insert(name, response);
    }

    fn page(&mut self, request: &str) -> Option<&[u8]> {
        if self.page_cache.contains_key(request) {
            Some(self.page_cache.get(request).unwrap())
        } else {
            let page = self.pages.get(request)?;
            self.page_cache.insert(request.to_owned(), page.bytes());
            Some(&self.page_cache[request])
        }
    }
}

fn main() {
    let mut pages = PageStore::new();

    println!("reading pages");
    for item in std::fs::read_dir("pages").unwrap() {
        let item = item.unwrap();
        if item.metadata().unwrap().is_file()
            && item.file_name().to_str().unwrap().ends_with(".fml")
        {
            println!("{}", item.file_name().to_str().unwrap());
            pages.add_page(
                item.file_name().into_string().unwrap(),
                response_from_file(item.path()).unwrap(),
            );
        }
    }

    let listener = TcpListener::bind("0.0.0.0:11121").unwrap();
    println!(
        "listening at {}. run this binary from froggi-server dir!",
        listener.local_addr().unwrap()
    );

    let page_store = Arc::new(Mutex::new(pages));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new client");
                let page_store = Arc::clone(&page_store);
                std::thread::spawn(move || handle_client(stream, page_store));
            }
            Err(e) => {
                println!("error {}", e);
            }
        }
    }
}

fn response_from_file(path: impl AsRef<std::path::Path>) -> Result<response::Response> {
    // TODO this is kind of garbage

    let path = path.as_ref();
    let data =
        std::fs::read_to_string(&path).context(format!("could not read '{}'", path.display()))?;

    let page = markup::parse::parse(&data).map_err(|mut errs| errs.pop().unwrap())?;

    let item_names = page.item_names();

    let mut item_data = Vec::new();
    for name in item_names.iter() {
        item_data.push(
            std::fs::read(path.parent().unwrap().join(name))
                .context(format!("could not read file {}", name))?,
        );
    }

    let items = item_names
        .into_iter()
        .zip(item_data.into_iter())
        .map(|(name, data)| {
            // TODO item kind
            response::Item::new(name, response::ItemKind::Png, data)
        })
        .collect();

    Ok(response::Response::new(
        response::ResponseKind::Page,
        data,
        items,
    )?)
}
