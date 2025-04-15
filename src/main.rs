#[allow(unused_imports)]
use std::net::TcpListener;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
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
                let file_path = path.trim_start_matches("/files/");
                if let Ok(file_content) = fs::read_to_string(Path::new("/tmp").join(file_path)) {
                    response_body = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        file_content.len(),
                        file_content
                    )
                    .into_bytes();
                } else {
                    response_body = "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes().to_vec();
                }
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
            let value: String = parts.next().unwrap().to_string();
            headers.push((key, value));
        }

        let mut body = String::new();
        if let Some(content_length) = headers
            .iter()
            .find(|&header| header.0 == String::from("Content-Length"))
        {
            let mut len = content_length.1.parse().unwrap();
            if len > 0 {
                let mut body_buf = vec![0; len];
                loop {
                    let line = lines.next().unwrap().unwrap();
                    if len >= line.as_bytes().len() {
                        body_buf.extend_from_slice(line.as_bytes());
                        len -= line.as_bytes().len();
                    } else {
                        body_buf.extend_from_slice(&line.as_bytes()[..len]);
                        break;
                    }
                }
                body = String::from_utf8(body_buf).unwrap();
            }
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
