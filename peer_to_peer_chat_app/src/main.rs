use std::env;
use std::io::{self, Write};
use std::net::UdpSocket;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("./p2p serve/client");
        return;
    }

    match args.get(1).map(|s| s.as_str()) {
        Some("serve") => connect_server(),
        Some("client") => connect_client(),
        _ => eprintln!("./p2p serve/client"),
    }
}

fn connect_server() {
    run(UdpSocket::bind("0.0.0.0:7777").unwrap(), None);
}

fn connect_client() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    run(socket, Some("127.0.0.1:7777".to_string()));
}

fn run(socket: UdpSocket, target: Option<String>) {
    let socket_read = socket.try_clone().unwrap();

    let target = target.unwrap_or_else(|| {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buff = [0u8; 1024];
        let (len, addr) = socket.recv_from(&mut buff).unwrap();
        println!("[RECEIVED]: {}", String::from_utf8_lossy(&buff[..len]));
        format!("{}:{}", addr.ip(), addr.port())
    });

    thread::spawn(move || {
        let mut buff = [0u8; 1024];
        loop {
            let (len, _) = socket_read.recv_from(&mut buff).unwrap();
            println!("[RECEIVED]: {}", String::from_utf8_lossy(&buff[..len]));
            print!("> ");
            io::stdout().flush().unwrap();
        }
    });

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        socket.send_to(input.trim().as_bytes(), &target).unwrap();
    }
}
