// Uncomment this block to pass the first stage
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";
const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 Not Found\r\n\r\n";

#[derive(Debug)]
#[allow(dead_code)]
struct Request {
    method: String,
    path: String,
    http_version: String,
    host: String,
    agent: String,
}

fn parse_requests(stream: &mut TcpStream) -> Request {
    let buf = &mut vec![0; 1024];
    stream.read(buf).unwrap(); // reads the data to the buffer
    let req_str = String::from_utf8_lossy(buf);
    println!("printing cow string\n {} ", req_str); // println takes no ownership and uses references
    let req: Vec<Vec<&str>> = req_str
        .split("\r\n")
        .map(|line| line.split(" ").collect())
        .collect();
    let rest: Vec<&str> = req[1..req.len() - 2]
        .iter()
        .map(|x| match x.get(1) {
            Some(s) => s.to_owned(),
            None => "",
        })
        .collect();

    let data: Vec<&str> = match req.get(0) {
        Some(s) => s.to_owned(),
        None => Vec::new(),
    }; // only first values are different

    println!("{:?} {:?}", data, rest);

    let host = match rest.get(0) {
        Some(s) => s,
        None => "",
    };

    let agent = match rest.get(1) {
        Some(s) => s,
        None => "",
    };

    // return a request
    Request {
        method: data.get(0).unwrap().to_string(),
        path: data.get(1).unwrap().to_string(),
        http_version: data.get(2).unwrap().to_string(),
        host: host.to_string(),
        agent: agent.to_string(),
    }
}

fn make_response(content: &str) -> String {
    let con = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        content.len(),
        content
    );
    con
}

fn main() {
    println!("logs appear here");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        // concurrent connections
        thread::spawn(move || match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let req = parse_requests(&mut stream);
                println!("{:?}", req);
                // now all are parsed in the struct according to requirements
                let path_split: Vec<&str> = req.path.as_str().split("/").collect();
                let resource = path_split.get(1).unwrap();
                let mut path = String::from("/");
                path.push_str(&resource);
                match path.as_str() {
                    "/" => stream.write_all(OK_RESPONSE.as_bytes()).unwrap(),
                    "/echo" => stream
                        .write_all(make_response(path_split[2..].join("/").as_str()).as_bytes())
                        .unwrap(),
                    "/user-agent" => stream
                        .write_all(make_response(&req.agent).as_bytes())
                        .unwrap(),
                    _ => stream.write_all(NOT_FOUND_RESPONSE.as_bytes()).unwrap(),
                }
            }
            Err(e) => {
                println!("error something wrong {}", e);
            }
        });
    }
}
