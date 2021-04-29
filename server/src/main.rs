use anyhow::{Context, Result};
use froggi::request::Request;
use froggi::response::{Item, ItemKind, Response, ResponseBuilder, ResponseKind};

use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream, page_store: &PageStore) {
    let request = Request::from_bytes(&mut stream).unwrap();

    println!("request: {:?}", request);

    match page_store.page(request.request()) {
        Some(page) => stream.write_all(page).unwrap(),
        None => stream.write_all(page_store.not_found()).unwrap(),
    }
}

struct PageStore {
    page_cache: HashMap<String, Vec<u8>>,
    not_found: Vec<u8>,
}

impl PageStore {
    fn new() -> PageStore {
        PageStore {
            page_cache: HashMap::new(),
            not_found: ResponseBuilder::default()
                .page(String::from("('not found')"))
                .kind(ResponseKind::Error)
                .build()
                .unwrap()
                .bytes(),
        }
    }

    fn add_page(&mut self, name: String, response: Response) {
        self.page_cache.insert(name, response.bytes());
    }

    fn page(&self, request: &str) -> Option<&Vec<u8>> {
        self.page_cache.get(request)
    }

    fn not_found(&self) -> &[u8] {
        &self.not_found
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

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new client");
                crossbeam::scope(|s| {
                    s.spawn(|_| {
                        handle_client(stream, &pages);
                    });
                })
                .unwrap();
            }
            Err(e) => {
                println!("error {}", e);
            }
        }
    }
}

fn response_from_file(path: impl AsRef<std::path::Path>) -> Result<Response> {
    // TODO this is kind of garbage

    let path = path.as_ref();
    let data =
        std::fs::read_to_string(&path).context(format!("could not read '{}'", path.display()))?;

    let page = froggi::markup::parse::parse(&data).map_err(|mut errs| errs.pop().unwrap())?;

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
        .map(|(name, data)| Item::new(name, ItemKind::Image, data))
        .collect();

    ResponseBuilder::default()
        .page(data)
        .items(items)
        .build()
        .map_err(|e| anyhow::anyhow!(e))
}
