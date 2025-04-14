use std::io::{BufRead, BufReader, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let buf_reader = BufReader::new(&mut stream);
                let request_line = buf_reader.lines().next().unwrap().unwrap();
                let path = request_line.split_whitespace().nth(1).unwrap();
                let response_body: Vec<u8>;
                match path {
                    "/" => response_body = "HTTP/1.1 200 OK\r\n\r\n".as_bytes().to_vec(),
                    path if path.starts_with("/echo/") => {
                        let str = path.trim_start_matches("/echo/");
                        response_body = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", str.len(), str).into_bytes();
                    }
                    _ => response_body = "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes().to_vec(),
                }
                stream.write_all(response_body.as_slice()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
