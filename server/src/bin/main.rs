use log::{error, info};
use log4rs;
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
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    println!("{}", "*".repeat(60));
    println!("Web server started now. Listening on http://127.0.0.1:7878");
    println!("{}", "*".repeat(60));
    // Accept incoming connections and handle each connection in a loop
    for stream in listener.incoming() {
        // Get the TcpStream from the incoming connection
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn is_allowed_ip(peer_ip: &str) -> bool {
    let allowed_ips = vec!["127.0.0.1"];
    allowed_ips.contains(&peer_ip)
}

fn is_allowed_user(username: &str) -> bool {
    let allowed_users = vec!["admin"];
    allowed_users.contains(&username)
}

fn handle_connection(mut stream: TcpStream) {
    let peer_ip = stream.peer_addr().unwrap().ip().to_string();

    let mut buffer = [0; 1024];
    if let Ok(_) = stream.read(&mut buffer) {
        let request = String::from_utf8_lossy(&buffer);
        let mut error_occurred = false;

        let (status_line, filename, content_type, username) =
            if request.starts_with("GET / HTTP/1.1\r\n") {
                ("HTTP/1.1 200 OK", "index.html", "text/html", None)
            } else if request.starts_with("GET /health HTTP/1.1\r\n") {
                ("HTTP/1.1 200 OK", "health.json", "application/json", None)
            } else if request.starts_with("GET /api/shipping/orders HTTP/1.1\r\n") {
                (
                    "HTTP/1.1 200 OK",
                    "shipping_orders.json",
                    "application/json",
                    None,
                )
            } else if request.starts_with("GET /sleep HTTP/1.1\r\n") {
                thread::sleep(Duration::from_secs(5));
                ("HTTP/1.1 200 OK", "index.html", "text/html", None)
            } else if let Some(captures) = regex::Regex::new(r"^GET /(\w+) HTTP/1.1\r\n")
                .unwrap()
                .captures(&request)
            {
                let username = captures.get(1).map(|m| m.as_str());
                if let Some(username) = username {
                    if is_allowed_user(username) {
                        ("HTTP/1.1 200 OK", "index.html", "text/html", Some(username))
                    } else {
                        (
                            "HTTP/1.1 403 Forbidden",
                            "access_denied.html",
                            "text/html",
                            None,
                        )
                    }
                } else {
                    error_occurred = true;
                    ("HTTP/1.1 400 Bad Request", "400.html", "text/html", None)
                }
            } else {
                error_occurred = true;
                ("HTTP/1.1 404 NOT FOUND", "404.html", "text/html", None)
            };

        let contents = fs::read_to_string(filename).unwrap_or_else(|_| {
            error_occurred = true;
            "File not found".to_string()
        });

        let response = if let Some(username) = username {
            if is_allowed_user(username) {
                let index_contents = fs::read_to_string("index.html").unwrap_or_else(|_| {
                    error_occurred = true;
                    "Index file not found".to_string()
                });

                format!(
                    "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                    status_line,
                    content_type,
                    index_contents.len(),
                    index_contents
                )
            } else {
                let access_denied_contents = fs::read_to_string("access_denied.html")
                    .unwrap_or_else(|_| {
                        error_occurred = true;
                        "Access denied file not found".to_string()
                    });

                format!(
                    "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                    "HTTP/1.1 403 Forbidden",
                    "text/html",
                    access_denied_contents.len(),
                    access_denied_contents
                )
            }
        } else {
            format!(
                "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                status_line,
                content_type,
                contents.len(),
                contents
            )
        };

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();

        if error_occurred {
            error!("Resource not found for request from: {}", peer_ip);
        } else {
            info!("Request received from: {}", peer_ip);
        }
    }
}
