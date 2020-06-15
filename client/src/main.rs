fn main() {
    froggi::hello();
    println!("connecting");
    froggi::send_request(include_str!("../server_address").trim(), "index.fml").unwrap();
    println!("sent");
}
