use std::{
    io::{Read, Write},
    net::TcpListener,
};

pub enum HTTPStatus {
    Ok(String, String),
    NotFound,
}

impl HTTPStatus {
    pub fn content_type(&self) -> String {
        match self {
            HTTPStatus::Ok(_, content_type) => content_type.clone(),
            HTTPStatus::NotFound => "text/plain".to_string(),
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            HTTPStatus::Ok(body, _) => {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                    self.content_type(),
                    body.len(),
                    body
                )
            }
            HTTPStatus::NotFound => {
                format!(
                    "HTTP/1.1 404 NOT FOUND\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                    self.content_type(),
                    0,
                    ""
                )
            }
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

                        let response = match request.split_whitespace().nth(1) {
                            Some("/") => HTTPStatus::Ok(
                                "Hello, world!".to_string(),
                                "text/plain".to_string(),
                            ),
                            Some("/user-agent") => {
                                let user_agent = request
                                    .split("\r\n")
                                    .find(|line| line.starts_with("User-Agent"))
                                    .map(|line| line.split(": ").nth(1).unwrap_or("Unknown"))
                                    .unwrap_or("Unknown")
                                    .to_string();

                                println!("User-Agent: {}", user_agent);
                                HTTPStatus::Ok(user_agent, "text/plain".to_string())
                            }
                            Some(path) if path.starts_with("/echo/") => {
                                let echo_string = &path[6..];

                                println!("Echoing: {}", echo_string);
                                HTTPStatus::Ok(echo_string.to_string(), "text/plain".to_string())
                            }
                            _ => HTTPStatus::NotFound,
                        };

                        match tcp_stream.write(response.to_string().as_bytes()) {
                            Ok(_) => {
                                println!("Response sent!");
                            }
                            Err(e) => {
                                println!("Failed to send response: {}", e);
                            }
                        }

                        match tcp_stream.flush() {
                            Ok(_) => {
                                println!("Flushed the stream!");
                            }
                            Err(e) => {
                                println!("Failed to flush the stream: {}", e);
                            }
                        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_ok() {
        let status = HTTPStatus::Ok("Hello, world!".to_string(), "text/plain".to_string());
        assert_eq!(status.content_type(), "text/plain");
    }

    #[test]
    fn test_content_type_not_found() {
        let status = HTTPStatus::NotFound;
        assert_eq!(status.content_type(), "text/plain");
    }
}
