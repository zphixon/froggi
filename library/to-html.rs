use std::fs::File;
use std::io::{Read, Write};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("no filename");
        return;
    }

    let mut file = File::open(&args[1]).unwrap();
    let mut page = String::new();
    file.read_to_string(&mut page).unwrap();
    let page = froggi::markup::parse::parse(&page).unwrap();

    let mut file = File::create("page.html").unwrap();
    file.write_all(froggi::markup::to_html(&page).as_bytes())
        .unwrap();

    println!("page ok");
}
