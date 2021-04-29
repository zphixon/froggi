use froggi::request::RequestKind;

fn main() {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or(String::from("127.0.0.1:11121"));

    let request = std::env::args().nth(2).unwrap_or(String::from(""));

    println!("connecting to {}, asking for '{}'", addr, request);

    let result = froggi::send_request(addr, &request, RequestKind::PageOnly).unwrap();

    match result.parse() {
        Ok(page) => {
            let document = froggi::markup::document::Document::from_page(&page);
            println!("{}", document.to_string());
        }

        Err(errors) => {
            for error in errors {
                println!("{}", error);
            }
        }
    }
}
