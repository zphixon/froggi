use std::fs::File;
use std::io::Read;

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

    dbg!(page);
    println!("page ok");
}
