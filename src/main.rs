use std::{
    io::{Read, Write},
    net::TcpListener,
};

enum HTTPStatus {
    Ok,
    NotFound,
}

impl HTTPStatus {
    fn to_string(&self) -> String {
        match self {
            HTTPStatus::Ok => "HTTP/1.1 200 OK\r\n\r\n".to_string(),
            HTTPStatus::NotFound => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
        }
    }
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut tcp_stream) => {
                println!("Connection established!");
                let mut buffer = [0; 1024];

                match tcp_stream.read(&mut buffer) {
                    Ok(0) => {
                        println!("Client closed the connection.");
                        continue;
                    }
                    Ok(_) => {
                        let request = String::from_utf8_lossy(&buffer[..]);
                        println!("Received request: {}", request);

                        let response_200 = HTTPStatus::Ok.to_string();
                        let response_404 = HTTPStatus::NotFound.to_string();

                        let response = match request.split_whitespace().nth(1) {
                            Some("/") => response_200,
                            _ => response_404,
                        };

                        tcp_stream.write(response.as_bytes()).unwrap();
                        tcp_stream.flush().unwrap();
                    }
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
