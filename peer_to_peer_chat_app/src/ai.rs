use std::env;
use std::io::{self, BufRead, Write};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const HELLO_INTERVAL_MS: u64 = 1_000;
const KEEPALIVE_MS: u64 = 5_000;
const PEER_TIMEOUT_SECS: u64 = 15;
const MAX_PACKET: usize = 4096;

fn encode_hello(name: &str) -> String {
    format!("HELLO|{}\n", name)
}
fn encode_msg(name: &str, text: &str) -> String {
    format!("MSG|{}|{}\n", name, text)
}
fn encode_bye(name: &str) -> String {
    format!("BYE|{}\n", name)
}

enum Packet {
    Hello(String),
    Msg { from: String, text: String },
    Bye(String),
    Unknown,
}

fn decode(raw: &str) -> Packet {
    let parts: Vec<&str> = raw.trim().splitn(3, '|').collect();
    match parts.as_slice() {
        ["HELLO", name] => Packet::Hello(name.to_string()),
        ["MSG", from, text] => Packet::Msg {
            from: from.to_string(),
            text: text.to_string(),
        },
        ["BYE", name] => Packet::Bye(name.to_string()),
        _ => Packet::Unknown,
    }
}

struct State {
    peer_name: Option<String>,
    last_seen: Option<Instant>,
    connected: bool,
}

impl State {
    fn new() -> Self {
        State {
            peer_name: None,
            last_seen: None,
            connected: false,
        }
    }

    fn touch(&mut self, name: &str) {
        self.peer_name = Some(name.to_string());
        self.last_seen = Some(Instant::now());
        self.connected = true;
    }

    fn timed_out(&self) -> bool {
        self.last_seen
            .map(|t| t.elapsed().as_secs() >= PEER_TIMEOUT_SECS)
            .unwrap_or(false)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("serve") => cmd_serve(&args),
        Some("connect") => cmd_connect(&args),
        _ => print_usage(),
    }
}

fn cmd_serve(args: &[String]) {
    if args.len() < 5 {
        print_usage();
        return;
    }

    let my_ip = parse_ip(&args[2]);
    let my_port = parse_port(&args[3]);
    let name = args[4..].join(" ");

    let socket = UdpSocket::bind(SocketAddr::new(my_ip, my_port)).expect("failed to bind");

    println!("=== Rust UDP P2P Chat ===");
    println!("Name   : {}", name);
    println!("Listen : {}:{}", my_ip, my_port);
    println!("Waiting for a peer...\n");

    let mut buf = [0u8; MAX_PACKET];
    let peer_addr = loop {
        let (n, src) = socket.recv_from(&mut buf).expect("recv failed");
        let raw = std::str::from_utf8(&buf[..n])
            .unwrap_or("")
            .trim()
            .to_string();
        if let Packet::Hello(peer_name) = decode(&raw) {
            println!("{} connected from {}\n", peer_name, src);
            break src;
        }
    };

    run_chat(socket, peer_addr, name);
}

fn cmd_connect(args: &[String]) {
    if args.len() < 7 {
        print_usage();
        return;
    }

    let my_ip = parse_ip(&args[2]);
    let my_port = parse_port(&args[3]);
    let peer_ip = parse_ip(&args[4]);
    let peer_port = parse_port(&args[5]);
    let name = args[6..].join(" ");

    let socket = UdpSocket::bind(SocketAddr::new(my_ip, my_port)).expect("failed to bind");
    let peer_addr = SocketAddr::new(peer_ip, peer_port);

    println!("=== Rust UDP P2P Chat ===");
    println!("Name   : {}", name);
    println!("Listen : {}:{}", my_ip, my_port);
    println!("Peer   : {}:{}", peer_ip, peer_port);
    println!("Connecting...\n");

    let hello = encode_hello(&name);
    socket
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();
    let mut buf = [0u8; MAX_PACKET];
    loop {
        let _ = socket.send_to(hello.as_bytes(), peer_addr);
        match socket.recv_from(&mut buf) {
            Ok((n, _)) => {
                let raw = std::str::from_utf8(&buf[..n])
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if let Packet::Hello(peer_name) = decode(&raw) {
                    println!("{} is reachable!\n", peer_name);
                    break;
                }
            }
            Err(_) => {
                print!(".");
                io::stdout().flush().unwrap();
            }
        }
    }
    socket.set_read_timeout(None).unwrap();

    run_chat(socket, peer_addr, name);
}

fn run_chat(socket: UdpSocket, peer_addr: SocketAddr, name: String) {
    println!("Commands: /quit\n");

    let socket = Arc::new(socket);
    let state = Arc::new(Mutex::new(State::new()));

    let (ui_tx, ui_rx) = mpsc::channel::<String>();

    let sock_r = Arc::clone(&socket);
    let state_r = Arc::clone(&state);
    let ui_tx_r = ui_tx.clone();
    thread::spawn(move || {
        let mut buf = [0u8; MAX_PACKET];
        loop {
            match sock_r.recv_from(&mut buf) {
                Ok((n, _)) => {
                    let raw = std::str::from_utf8(&buf[..n])
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    match decode(&raw) {
                        Packet::Hello(pname) => {
                            let mut s = state_r.lock().unwrap();
                            let was = s.connected;
                            s.touch(&pname);
                            if !was {
                                let _ = ui_tx_r.send(format!("[{} is online]", pname));
                            }
                        }
                        Packet::Msg { from, text } => {
                            state_r.lock().unwrap().touch(&from);
                            let _ = ui_tx_r.send(format!("{}: {}", from, text));
                        }
                        Packet::Bye(pname) => {
                            let _ = ui_tx_r.send(format!("[{} disconnected]", pname));
                            std::process::exit(0);
                        }
                        Packet::Unknown => {}
                    }
                }
                Err(_) => break,
            }
        }
    });

    thread::spawn(move || {
        for line in ui_rx {
            println!("\r{}", line);
            print!("> ");
            io::stdout().flush().unwrap();
        }
    });

    let sock_k = Arc::clone(&socket);
    let state_k = Arc::clone(&state);
    let name_k = name.clone();
    thread::spawn(move || {
        let hello = encode_hello(&name_k);
        loop {
            thread::sleep(Duration::from_millis(KEEPALIVE_MS));
            let _ = sock_k.send_to(hello.as_bytes(), peer_addr);
            let s = state_k.lock().unwrap();
            if s.connected && s.timed_out() {
                println!("[{} timed out]", s.peer_name.clone().unwrap_or_default());
                std::process::exit(1);
            }
        }
    });

    let sock_h = Arc::clone(&socket);
    let state_h = Arc::clone(&state);
    let name_h = name.clone();
    thread::spawn(move || {
        let hello = encode_hello(&name_h);
        loop {
            thread::sleep(Duration::from_millis(HELLO_INTERVAL_MS));
            if state_h.lock().unwrap().connected {
                break;
            }
            let _ = sock_h.send_to(hello.as_bytes(), peer_addr);
        }
    });

    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match stdin.lock().read_line(&mut input) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        let text = input.trim().to_string();
        if text.is_empty() {
            continue;
        }

        if text == "/quit" {
            let _ = socket.send_to(encode_bye(&name).as_bytes(), peer_addr);
            break;
        }

        if socket
            .send_to(encode_msg(&name, &text).as_bytes(), peer_addr)
            .is_err()
        {
            eprintln!("send failed");
            break;
        }
        println!("\ryou: {}", text);
    }

    println!("Goodbye!");
}

fn parse_ip(s: &str) -> IpAddr {
    s.parse().unwrap_or_else(|_| {
        eprintln!("Invalid IP: {}", s);
        std::process::exit(1);
    })
}

fn parse_port(s: &str) -> u16 {
    s.parse().unwrap_or_else(|_| {
        eprintln!("Invalid port: {}", s);
        std::process::exit(1);
    })
}

fn print_usage() {
    println!("Usage:");
    println!("  serve   <my-ip> <my-port> <name>");
    println!("  connect <my-ip> <my-port> <peer-ip> <peer-port> <name>");
    println!();
    println!("Examples:");
    println!("  cargo run -- serve 0.0.0.0 7000 Alice");
    println!("  cargo run -- connect 0.0.0.0 7001 127.0.0.1 7000 Bob");
}
