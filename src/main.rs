// Uncomment this block to pass the first stage
use std::io::Write;
use std::net::{TcpListener, TcpStream};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                handle_success_connect(_stream);
                println!("successful connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_success_connect(mut _stream: TcpStream) {
    let response = format!("HTTP/1.1 200 OK\r\n\r\n");
    _stream.write_all(response.as_bytes()).unwrap();
}
