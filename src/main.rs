use {std::fs::read};
#[allow(dead_code)]
use {
    std::{
        collections::HashMap, 
        io::{BufRead, BufReader, Error, Lines, Write}, 
        net::{TcpListener, TcpStream},
        fs,
        env
    },
};

mod threadpool;

#[derive(Default)]
struct HttpRequest {
    method: String,
    target: String,
    version: String,
    headers: HashMap<String, String>
}

impl HttpRequest {
    fn new() -> Self {
        Self::default()
    }
    
    fn parse_request(&mut self, request: BufReader<&TcpStream>) {
        let mut request_lines = request.lines();
        let binding = request_lines.next().unwrap().unwrap();
        let mut info_line = binding.split_whitespace();
        
        self.method = info_line.next().unwrap().to_string();
        self.target = info_line.next().unwrap().to_string();
        self.version = info_line.next().unwrap().to_string();
        self.headers = HttpRequest::parse_request_headers(self, request_lines);
    }
    
    fn parse_request_headers(&mut self, headers: Lines<BufReader<&TcpStream>>) -> HashMap<String, String>{
        let mut map: HashMap<String, String> = HashMap::new();
        for head in headers {
            let header = Ok::<String, Error>(head.unwrap()).unwrap();
            if !header.is_empty() {
                let mut parts = header.split_whitespace();
                map.insert(parts.next().unwrap().to_string().strip_suffix(":").unwrap().to_lowercase(), parts.next().unwrap().to_string());
            } else {
                break;
            } 
        }
        println!("");
        map
    }
}

fn main() -> std::io::Result<()> {
    println!("{}", env::current_dir().unwrap().to_str().unwrap());
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let thread_pool = threadpool::ThreadPool::new(200);
            
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                print!("accepted new connection ");
                thread_pool.execute(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    
    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let mut http_request = HttpRequest::new();
    http_request.parse_request(buf_reader);
    
    let mut target = http_request.target.as_str();
    match target {
        "/" => {
            let file_body = fs::read("/home/michael-lindsay/simple-http-server-rust/file.html").unwrap();
            let response_body = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length:{}\r\n\r\n", file_body.len());
            let _ = stream.write([response_body.as_bytes(), file_body.as_slice()].concat().as_slice());
        }
        target if target.starts_with("/echo") => {
            let mut echo_path = target.strip_prefix("/echo").unwrap();
            if echo_path.starts_with("/") {echo_path = echo_path.strip_prefix("/").unwrap();}
            let _ = stream.write(
                format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", echo_path.len(), echo_path)
                .as_bytes()
            );
        },
        target if target.starts_with("/files") => {
            let file_path = format!("/home/michael-lindsay/simple-http-server-rust/{}", target.strip_prefix("/files/").unwrap());
            if fs::exists(file_path).unwrap() {
                let file_body = fs::read(format!("/home/michael-lindsay/simple-http-server-rust/{}", target.strip_prefix("/files/").unwrap())).ok().unwrap();
                let response_body = match target {
                    target if target.ends_with(".html") => format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length:{}\r\n\r\n", file_body.len()),
                    _ => format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length:{}\r\n\r\n", file_body.len())
                };
                let _ = stream.write([response_body.as_bytes(), file_body.as_slice()].concat().as_slice());                
            } else {
                let _ = stream.write(b"HTTP/1.1 404 NOT FOUND\r\n\r\n");
            }
        }
        _ => {
            target = target.strip_prefix("/").unwrap();
            if http_request.headers.contains_key(target) {
                let _ = stream.write(
                    format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", target.len(), http_request.headers.get(target).unwrap())
                    .as_bytes()
                );
            } else {
                let _ = stream.write(b"HTTP/1.1 404 NOT FOUND\r\n\r\n");
            }
        }
    };
    
    stream.flush().unwrap();
}