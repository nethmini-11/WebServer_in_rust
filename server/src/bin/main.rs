use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use server::ThreadPool;

fn main() {
    // Create a TCP listener and bind it to the specified address and port
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let pool = ThreadPool::new(4);

    // Accept incoming connections and handle each connection in a loop
    for stream in listener.incoming() {
        // Get the TcpStream from the incoming connection
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024]; // Create a mutable buffer of size 1024 bytes

    stream.read(&mut buffer).unwrap(); // Read data from the stream and store it in the buffer

    let get = b"GET / HTTP/1.1\r\n";
    let health = b"GET /health HTTP/1.1\r\n";
    let api_shipping_orders = b"GET /api/shipping/orders HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename, content_type) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html", "text/html")
    } else if buffer.starts_with(health) {
        ("HTTP/1.1 200 OK", "health.json", "application/json")
    } else if buffer.starts_with(api_shipping_orders) {
        ("HTTP/1.1 200 OK", "shipping_orders.json", "application/json")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html", "text/html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html", "text/html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content_type,
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
