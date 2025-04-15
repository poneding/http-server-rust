#[allow(unused_imports)]
use std::net::TcpListener;
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let response_body: Vec<u8>;
                if let Some(request) = Request::from_tcp_stream(&mut stream) {
                    match request.path.as_str() {
                        "/" => response_body = "HTTP/1.1 200 OK\r\n\r\n".as_bytes().to_vec(),
                        path if path.starts_with("/echo/") => {
                            let body = path.trim_start_matches("/echo/");
                            response_body = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", body.len(), body).into_bytes();
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
                            response_body=format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", body.len(), body).into_bytes();
                        }
                        _ => response_body = "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes().to_vec(),
                    }
                } else {
                    response_body = "HTTP/1.1 200 OK".as_bytes().to_vec();
                }
                stream.write_all(response_body.as_slice()).unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
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
        let buf_reader = BufReader::new(stream);
        let mut lines = buf_reader.lines();

        let request_line = lines.next().unwrap().unwrap();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap().to_string();
        let path = parts.next().unwrap().to_string();
        let version = parts.next().unwrap().to_string();

        let mut headers = Vec::new();
        loop {
            let line = lines.next().unwrap().unwrap();
            if line.is_empty() {
                // 遇到空行，表示请求头结束
                break;
            }

            let mut parts = line.split(": ");
            let key = parts.next().unwrap().to_string();
            let value = parts.next().unwrap().to_string();
            headers.push((key, value));
        }

        // TODO: 处理请求体
        Some(Self {
            method,
            path,
            version,
            headers,
            body: String::new(),
        })
    }

    fn from_raw_request(raw_request: &str) -> Option<Self> {
        let mut lines = raw_request.lines();
        let request_line = lines.next().unwrap();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap().to_string();
        let path = parts.next().unwrap().to_string();
        let version = parts.next().unwrap().to_string();

        let mut headers = Vec::new();
        while let Some(line) = lines.next() {
            if line.is_empty() {
                // 遇到空行，表示请求头结束
                break;
            }

            let mut parts = line.split(": ");
            let key = parts.next().unwrap().to_string();
            let value = parts.next().unwrap().to_string();
            headers.push((key, value));
        }

        let body = lines.collect::<Vec<_>>().join("\n");

        Some(Self {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Request;

    #[test]
    fn test_init_request_from_tcp_stream() {
        let raw_request = "POST /users HTTP/1.1\r\nHost: users.api.com\r\nUser-Agent: curl/7.68.0\r\nAccept: */*\r\n\r\n{\"id\": 1, \"name\": \"dp\"}";

        if let Some(request) = Request::from_raw_request(raw_request) {
            assert_eq!(request.method, "POST");
            assert_eq!(request.path, "/users");
            assert_eq!(request.version, "HTTP/1.1");

            assert_eq!(request.body, "{\"id\": 1, \"name\": \"dp\"}")
        } else {
            panic!()
        }
    }
}
