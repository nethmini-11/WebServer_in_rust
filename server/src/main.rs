use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {
    // Create a TCP listener and bind it to the specified address and port
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // Accept incoming connections and handle each connection in a loop
    for stream in listener.incoming() {
        // Get the TcpStream from the incoming connection
        let stream = stream.unwrap();

        handle_connection(stream)
    }
}
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024]; // Create a mutable buffer of size 1024 bytes

    stream.read(&mut buffer).unwrap(); // Read data from the stream and store it in the buffer

    let get =b"GET /HTTP/1.1\r\n";
    if buffer.starts_with(get){
        let contents = fs::read_to_string("index.html").unwrap();
   
        let response = format!(
         "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
     );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }else{
        let status_line ="HTTP/1.1 404 NOT FOUND";
        let contents=fs::read_to_string("404.html").unwrap();

        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
           contents.len(),
           contents
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

   
}
