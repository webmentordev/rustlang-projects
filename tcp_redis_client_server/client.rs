use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

struct RedisClient {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl RedisClient {
    fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        let reader = BufReader::new(stream.try_clone()?);

        Ok(RedisClient { stream, reader })
    }

    fn send_command(&mut self, command: &str) -> io::Result<String> {
        writeln!(self.stream, "{}", command)?;
        self.stream.flush()?;

        let mut response = String::new();
        self.reader.read_line(&mut response)?;

        Ok(response.trim().to_string())
    }

    fn run(&mut self) -> io::Result<()> {
        println!("Connected to Redis server. Type 'QUIT' to exit.");

        loop {
            print!("redis> ");
            io::stdout().flush()?;

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => break,
                Ok(_) => {
                    let command = input.trim();
                    if command.is_empty() {
                        continue;
                    }
                    if command.to_uppercase() == "QUIT" || command.to_uppercase() == "EXIT" {
                        println!("Bye!");
                        break;
                    }
                    match self.send_command(command) {
                        Ok(response) => println!("{}", response),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    match RedisClient::connect("127.0.0.1:6789") {
        Ok(mut client) => {
            client.run()?;
        }
        Err(e) => {
            eprintln!("Failed to connect to Redis server: {}", e);
            eprintln!("Make sure the server is running on 127.0.0.1:6789");
        }
    }

    Ok(())
}
