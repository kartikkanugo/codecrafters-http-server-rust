// Uncomment this block to pass the first stage
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::string;

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
    // read the data here important to preallocate else it reads till infinity
    let mut request_data_vec: Vec<u8> = vec![0; 1000];
    match _stream.read(request_data_vec.as_mut_slice()) {
        Ok(_) => {
            // check the request data vec
            println!(
                "buffer: {}",
                String::from_utf8_lossy(request_data_vec.as_slice())
            );

            // Now lets accept the string if it only contains / marker after the local host
            let string_data = String::from_utf8_lossy(request_data_vec.as_slice());
            // now get lines
            let line_data: Vec<&str> = string_data.split("\r\n").collect();
            // now in the first line there is get method or some method with some html link
            let start_line = line_data[0];
            let start_parts: Vec<&str> = start_line.split(" ").collect();
            let path = start_parts[1];
            let paths: Vec<&str> = path.split("/").collect();
            if paths[1] == "echo" {
                let response = format!("HTTP/1.1 200 OK\r\n\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{} \r\n", paths[2].len(),paths[2]);
                _stream.write_all(response.as_bytes()).unwrap();
            } else {
                let response = format!("HTTP/1.1 404 Not Found\r\n\r\n");
                _stream.write_all(response.as_bytes()).unwrap();
            }
        }
        Err(_err) => {
            // Nothing to do here
        }
    };

    // send the response here
}
