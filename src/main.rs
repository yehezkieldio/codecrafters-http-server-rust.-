use std::{
    error::Error,
    io::{Read, Write},
    net::TcpListener,
};

use tokio::{fs::File, io::AsyncWriteExt};

pub enum HTTPStatus {
    Ok(String, String),
    NotFound,
    Created,
}

impl HTTPStatus {
    pub fn content_type(&self) -> String {
        match self {
            HTTPStatus::Ok(_, content_type) => content_type.clone(),
            HTTPStatus::NotFound => "text/plain".to_string(),
            HTTPStatus::Created => "text/plain".to_string(),
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
            HTTPStatus::Created => {
                format!(
                    "HTTP/1.1 201 CREATED\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                    self.content_type(),
                    0,
                    ""
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

fn discover_files_from_directory(directory: &str) -> Vec<String> {
    let paths = std::fs::read_dir(directory).unwrap();
    let mut files = vec![];
    for path in paths {
        let path = path.unwrap().path();
        if path.is_file() {
            files.push(path.file_name().unwrap().to_str().unwrap().to_string());
        }
    }
    files
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Logs from your program will appear here!");
    let prestart_directory = std::env::args().nth(2);
    if prestart_directory.is_some() {
        let prestart_directory_value = match prestart_directory {
            Some(directory) => directory,
            None => ".".to_string(),
        };
        let files = discover_files_from_directory(&prestart_directory_value);
        println!("Discovered files from provided directory flag: {:?}", files);
    }

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    loop {
        let (mut tcp_stream, _) = listener.accept().unwrap();
        tokio::spawn(async move {
            println!("Connection established!");
            let mut buffer: [u8; 4096] = [0; 4096];
            let directory_flag = std::env::args().nth(2);

            match tcp_stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Client closed the connection.");
                    return;
                }
                Ok(_) => {
                    let request = String::from_utf8_lossy(&buffer[..]);
                    println!("Received request: {}", request);

                    let response = match request.split_whitespace().nth(1) {
                        Some("/") => {
                            HTTPStatus::Ok("Hello, world!".to_string(), "text/plain".to_string())
                        }
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
                        Some(path) if path.starts_with("/files/") => {
                            let file_name = &path[7..];
                            match directory_flag {
                                Some(directory) => {
                                    let file_path = format!("{}/{}", directory, file_name);

                                    if request.split_whitespace().nth(0) == Some("POST") {
                                        println!("POST request received!");

                                        let file_content =
                                            request.split("\r\n\r\n").nth(1).unwrap_or("");
                                        println!("File content: {}", file_content);

                                        let file_content = file_content
                                            .split("\r\n")
                                            .map(|line| line.trim())
                                            .collect::<Vec<&str>>()
                                            .join("\n");
                                        println!("File content: {}", file_content);

                                        match File::create(file_path).await {
                                            Ok(mut file) => {
                                                match file.write_all(file_content.as_bytes()).await
                                                {
                                                    Ok(_) => {
                                                        println!("File content written!");
                                                        HTTPStatus::Created
                                                    }
                                                    Err(e) => {
                                                        println!(
                                                            "Failed to write file content: {}",
                                                            e
                                                        );
                                                        HTTPStatus::NotFound
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!("Failed to create file: {}", e);
                                                HTTPStatus::NotFound
                                            }
                                        }
                                    } else {
                                        match tokio::fs::read_to_string(file_path).await {
                                            Ok(file_content) => HTTPStatus::Ok(
                                                file_content,
                                                "application/octet-stream".to_string(),
                                            ),
                                            Err(_) => HTTPStatus::NotFound,
                                        }
                                    }
                                }
                                None => HTTPStatus::NotFound,
                            }
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
        });
    }
}
