#[allow(unused_imports)]
use std::net::TcpListener;
use std::{
    env, fs,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    path::Path,
    thread, vec,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || handle_request(&mut stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_request(stream: &mut TcpStream) {
    let response_body: Vec<u8>;
    if let Some(request) = Request::from_tcp_stream(stream) {
        match request.path.as_str() {
            "/" => response_body = "HTTP/1.1 200 OK\r\n\r\n".as_bytes().to_vec(),
            path if path.starts_with("/echo/") => {
                let body = path.trim_start_matches("/echo/");
                response_body = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                )
                .into_bytes();
            }
            path if path.starts_with("/files/") => {
                response_body = handle_create_file_request(&request);
            }
            "/user-agent" => {
                let body;
                if let Some(user_agent) = request
                    .headers
                    .iter()
                    .find(|header| header.0 == "User-Agent")
                {
                    body = user_agent.clone().1;
                } else {
                    body = String::new();
                }
                response_body = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                )
                .into_bytes();
            }
            _ => response_body = "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes().to_vec(),
        }
    } else {
        response_body = "HTTP/1.1 200 OK".as_bytes().to_vec();
    }
    stream.write_all(response_body.as_slice()).unwrap();
    stream.flush().unwrap();
}

fn handle_create_file_request(request: &Request) -> Vec<u8> {
    let response: Vec<u8>;
    let file_name = request.path.trim_start_matches("/files/");

    let dir = env::args().nth(2).unwrap();
    let file_path = Path::new(&dir).join(file_name);
    match request.method.as_str() {
        "POST" => {
            fs::write(file_path, request.body.as_str()).unwrap();
            response = "HTTP/1.1 201 Created\r\n\r\n".as_bytes().to_vec();
        }
        "GET" => {
            if let Ok(file_content) = fs::read_to_string(file_path) {
                response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                        file_content.len(),
                        file_content
                    )
                    .into_bytes();
            } else {
                response = "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes().to_vec();
            }
        }
        _ => {
            response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n"
                .as_bytes()
                .to_vec();
        }
    }

    response
}
#[allow(dead_code)]
struct Request {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) version: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: String,
}

impl Request {
    fn from_tcp_stream(stream: &mut TcpStream) -> Option<Self> {
        let mut buf_reader = BufReader::new(stream);
        let mut request_line = String::new();
        buf_reader.read_line(&mut request_line).unwrap();
        // let mut lines = buf_reader.lines();

        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap().to_string();
        let path = parts.next().unwrap().to_string();
        let version = parts.next().unwrap().to_string();

        let mut headers = Vec::new();
        let mut content_length = 0;
        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line).unwrap();
            if line == "\r\n" || line == "\n" {
                // 空行，表示请求头结束
                break;
            }
            let line = line.trim_end();
            if let Some((key, value)) = line.split_once(": ") {
                headers.push((key.to_string(), value.to_string()));
                if key == "Content-Length" {
                    content_length = value.parse::<usize>().unwrap();
                }
            }
        }

        let mut body = String::new();
        if content_length > 0 {
            let mut body_buf = vec![0; content_length];
            buf_reader.read_exact(&mut body_buf).unwrap();
            body = String::from_utf8(body_buf).unwrap();
        }

        // TODO: 处理请求体
        Some(Self {
            method,
            path,
            version,
            headers,
            body: body,
        })
    }
}

#[cfg(test)]
mod tests {}
