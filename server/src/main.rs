use std::net::TcpListener;

fn main() {
    // Create a TCP listener and bind it to the specified address and port
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // Accept incoming connections and handle each connection in a loop
    for stream in listener.incoming() {
        // Get the TcpStream from the incoming connection
        let stream = stream.unwrap();

        
        println!("Connection established!");
        
    }
}
