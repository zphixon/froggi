use std::io::Write;

fn main() {
    let mut handles = Vec::new();

    for i in 0..100 {
        println!("spawn {}", i);

        handles.push(std::thread::spawn(move || {
            std::io::stdout()
                .lock()
                .write(format!("request {}\n", i).as_bytes())
                .unwrap();

            froggi::send_request(
                "localhost:11121",
                "test_markup.fml",
                froggi::request::RequestKind::Page,
            )
            .map(|r| {
                std::io::stdout()
                    .lock()
                    .write(format!("{} ok {}\n", i, r.id()).as_bytes())
            })
            .map_err(|e| {
                std::io::stdout()
                    .lock()
                    .write(format!("{} err {}\n", i, e).as_bytes())
            })
            .unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
