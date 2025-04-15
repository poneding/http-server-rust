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
    let response = match Request::from_tcp_stream(stream) {
        Some(request) => match request.path.as_str() {
            "/" => make_response(HTTPStatus::OK, None, &[]),
            path if path.starts_with("/echo/") => {
                let body = path.trim_start_matches("/echo/");

                make_response(HTTPStatus::OK, Some("text/plain"), body.as_bytes())
            }
            path if path.starts_with("/files/") => handle_create_file_request(&request),
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
                make_response(HTTPStatus::OK, Some("text/plain"), body.as_bytes())
            }
            _ => make_response(HTTPStatus::NotFound, None, &[]),
        },
        None => {
            // 处理请求失败的情况
            make_response(HTTPStatus::NotFound, None, &[])
        }
    };
    stream.write_all(response.as_slice()).unwrap();
    stream.flush().unwrap();
}

fn handle_create_file_request(request: &Request) -> Vec<u8> {
    let file_name = request.path.trim_start_matches("/files/");

    let dir = env::args().nth(2).unwrap();
    let file_path = Path::new(&dir).join(file_name);
    match request.method.as_str() {
        "POST" => {
            fs::write(file_path, request.body.as_str()).unwrap();
            make_response(HTTPStatus::Created, None, &[])
        }
        "GET" => {
            if let Ok(file_content) = fs::read_to_string(file_path) {
                make_response(
                    HTTPStatus::OK,
                    Some("application/octet-stream"),
                    file_content.as_bytes(),
                )
            } else {
                make_response(HTTPStatus::NotFound, None, &[])
            }
        }
        _ => make_response(HTTPStatus::MethodNotAllowed, None, &[]),
    }
}

enum HTTPStatus {
    OK,
    Created,
    NotFound,
    MethodNotAllowed,
}

impl HTTPStatus {
    fn to_string(&self) -> String {
        match self {
            HTTPStatus::OK => "200 OK".to_string(),
            HTTPStatus::Created => "201 Created".to_string(),
            HTTPStatus::NotFound => "404 Not Found".to_string(),
            HTTPStatus::MethodNotAllowed => "405 Method Not Allowed".to_string(),
        }
    }
}

fn make_response(status: HTTPStatus, content_type: Option<&str>, body: &[u8]) -> Vec<u8> {
    let headers = if let Some(content_type) = content_type {
        format!(
            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            status.to_string(),
            content_type,
            body.len()
        )
    } else {
        format!("HTTP/1.1 {}\r\n\r\n", status.to_string())
    };

    let mut response = headers.into_bytes();
    response.extend_from_slice(body);

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
            body,
        })
    }
}

#[cfg(test)]
mod tests {}
