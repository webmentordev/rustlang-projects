use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

struct RedisServer {
    listener: TcpListener,
    store: HashMap<String, String>,
}

impl RedisServer {
    fn new() -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:6789")?;
        println!("Redis server running at: 127.0.0.1:6789");
        Ok(RedisServer {
            listener,
            store: HashMap::new(),
        })
    }

    fn set(&mut self, key: String, value: String) -> String {
        self.store.insert(key, value);
        "OK".to_string()
    }

    fn del(&mut self, key: String) -> String {
        match self.store.remove(&key) {
            Some(_) => "1".to_string(),
            None => "0".to_string(),
        }
    }

    fn get(&self, key: &str) -> String {
        match self.store.get(key) {
            Some(value) => value.clone(),
            None => "(nil)".to_string(),
        }
    }

    fn process(&mut self, line: String) -> String {
        let args: Vec<&str> = line.trim().split_whitespace().collect();

        if args.is_empty() {
            return "ERROR: Empty command".to_string();
        }

        match args[0].to_uppercase().as_str() {
            "GET" => {
                if args.len() != 2 {
                    return "ERROR: GET requires exactly 1 argument".to_string();
                }
                self.get(args[1])
            }
            "SET" => {
                if args.len() < 3 {
                    return "ERROR: SET requires at least 2 arguments".to_string();
                }
                let key = args[1].to_string();
                let value = args[2..].join(" ");
                self.set(key, value)
            }
            "DEL" => {
                if args.len() != 2 {
                    return "ERROR: DEL requires exactly 1 argument".to_string();
                }
                self.del(args[1].to_string())
            }
            _ => {
                format!("ERROR: Unknown command '{}'", args[0])
            }
        }
    }

    fn handle_client(&mut self, mut stream: TcpStream) -> io::Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let response = self.process(line.clone());
                    writeln!(stream, "{}", response)?;
                    stream.flush()?;
                }
                Err(e) => {
                    eprintln!("Error reading from client: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }

    fn run(&mut self) -> io::Result<()> {
        loop {
            match self.listener.accept() {
                Ok((stream, _addr)) => {
                    println!("New client connected");
                    if let Err(e) = self.handle_client(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut server = RedisServer::new()?;
    server.run()
}
