fn main() {
    froggi::hello();
    println!("connecting");
    let result =
        froggi::send_request(include_str!("../server_address").trim(), "index.fml").unwrap();
    println!("got {:?}", result);
}
