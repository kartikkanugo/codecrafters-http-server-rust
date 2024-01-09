use std::fmt::{self};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
};

use itertools::Itertools;

enum HttpStatusCode {
    Ok = 200,
    NotFound = 404,
    NoContent = 201,
    InternalServerError = 500,
}

impl fmt::Display for HttpStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpStatusCode::Ok => write!(f, "200 OK"),
            HttpStatusCode::NotFound => write!(f, "404 NOT FOUND"),
            HttpStatusCode::NoContent => write!(f, "201 NO CONTENT"),
            HttpStatusCode::InternalServerError => write!(f, "500 INTERNAL SERVER ERROR"),
        }
    }
}

#[derive(Debug)]
enum HttpMethod {
    Get,
    Post,
}

impl FromStr for HttpMethod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            _ => Err(anyhow::anyhow!("error invalid methods")),
        }
    }
}

struct HttpRequest {
    method: HttpMethod,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}

impl FromStr for HttpRequest {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let first_line = lines.next().unwrap();
        let (method, path, version) = {
            let mut parts = first_line.split_whitespace();
            (
                parts
                    .next()
                    .unwrap()
                    .to_string()
                    .parse::<HttpMethod>()
                    .unwrap(),
                parts.next().unwrap().to_string(),
                parts.next().unwrap().to_string(),
            )
        };
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut body = String::new();

        for line in lines {
            if let Some((header, value)) = line.split_once(':') {
                headers.insert(
                    header.trim().to_lowercase().to_string(),
                    value.trim().to_string(),
                );
            } else if matches!(method, HttpMethod::Post) && !line.is_empty() {
                body.push_str(line);
            }
        }

        Ok(Self {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}

const LINE_ENDING: &str = "\r\n";

fn main() {
    println!("debug tested and connecting to port 4221");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                std::thread::spawn(move || {
                    println!("new connection thread");
                    stream_handler(&mut stream);
                });
            }
            Err(e) => {
                println!("error found {}", e);
            }
        }
    }
}

// http complete
fn stream_handler(mut stream: &mut TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream); // here pass the ownership of stream to unknown sized trait as we will not use it again but reference also can be passed
    let buffer = buf_reader.fill_buf().unwrap();
    let str = String::from_utf8_lossy(buffer);

    if let Ok(req) = str.parse::<HttpRequest>() {
        println!(
            "Req type: {:?} , path: {}, version: {}, body: {}",
            req.method, req.path, req.version, req.body
        );

        let mut paths = req.path.split("/");
        if let Some(path) = paths.next() {
            match path {
                "" => {
                    if let Some(path2) = paths.next() {
                        match path2 {
                            "echo" => {
                                let content = paths.collect_vec().join("/");
                                println!("string path obtained {}", content);
                                // print the content with 200 ok
                                status_ok_with_text(stream, content.as_str());
                            }
                            "user-agent" => {
                                let content = req.headers.get("user-agent").unwrap();
                                println!("string path obtained {}", content);
                                // print the content with 200 ok
                                status_ok_with_text(stream, content.as_str());
                            }
                            "files" => {
                                let filename = paths.collect_vec().join("/");
                                println!("string path obtained {}", filename);
                                if matches!(req.method, HttpMethod::Post) {
                                    save_received_file(filename.as_str(), req.body.as_str());
                                    status_no_content(stream);
                                } else if let Some(content) = get_file_content(filename.as_str()) {
                                    status_ok_with_octet_stream(stream, content.as_str());
                                } else {
                                    status_not_found(stream);
                                }
                            }
                            "" => {
                                // this is ok
                                status_ok(stream);
                            }
                            _ => {
                                // nothing as per requirements
                                println!("nothing found in path {}", path);
                                // not found response
                                status_not_found(stream);
                            }
                        }
                    } else {
                        // not found
                        status_not_found(stream);
                    }
                }
                _ => {
                    println!("not found path {}", path);
                    status_not_found(stream);
                }
            }
        }
    } else {
        println!("could not parse anything");
        status_internal_error(stream);
    }
}

fn get_file_path(filename: &str) -> String {
    let cmd_args: Vec<String> = std::env::args().collect();
    let dir_path = &cmd_args[2];
    println!("{:?}", cmd_args);
    format!("{}/{}", dir_path, filename)
}

fn get_file_content(filename: &str) -> Option<String> {
    if let Ok(contents) = std::fs::read_to_string(get_file_path(filename)) {
        Some(contents)
    } else {
        None
    }
}

fn status_ok_with_octet_stream(stream: &mut TcpStream, content: &str) {
    let response = create_response(HttpStatusCode::Ok, "application/octet-stream", content);
    send_response(stream, response.as_str())
}

fn save_received_file(file_name: &str, file_contents: &str) {
    std::fs::write(get_file_path(file_name), file_contents).unwrap();
}

fn status_ok_with_text(stream: &mut TcpStream, content: &str) {
    let response = create_response(HttpStatusCode::Ok, "text/plain", content);
    send_response(stream, response.as_str());
}

fn status_ok(stream: &mut TcpStream) {
    let response = create_response(HttpStatusCode::Ok, "", "");
    send_response(stream, response.as_str());
}

fn status_not_found(stream: &mut TcpStream) {
    let response = create_response(HttpStatusCode::NotFound, "", "");
    send_response(stream, response.as_str());
}

fn status_no_content(stream: &mut TcpStream) {
    let response = create_response(HttpStatusCode::NoContent, "", "");
    send_response(stream, response.as_str());
}

fn status_internal_error(stream: &mut TcpStream) {
    let response = create_response(HttpStatusCode::InternalServerError, "", "");
    send_response(stream, response.as_str());
}

fn create_response(status_code: HttpStatusCode, content_type: &str, content: &str) -> String {
    let mut response = String::from("");
    response.push_str("HTTP/1.1 ");
    response.push_str(format!("{}{}", status_code, LINE_ENDING).as_str());
    if !content.is_empty() {
        response.push_str(format!("Content-Type: {}{}", content_type, LINE_ENDING).as_str());
        response.push_str(format!("Content-Length: {}{}", content.len(), LINE_ENDING).as_str());
        response.push_str(LINE_ENDING);
        response.push_str(content);
    } else {
        response.push_str(LINE_ENDING);
        response.push_str(LINE_ENDING);
    }

    response
}

fn send_response(stream: &mut TcpStream, response: &str) {
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
