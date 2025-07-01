//! Simple UDP Ping utility in Rust

use std::env;
use std::net::UdpSocket;
use std::time::Duration;

const PORT: u16 = 34254;
const PING_MSG: &[u8] = b"PING";

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} [server|client]", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "server" => run_server(),
        "client" => run_client(),
        _ => {
            eprintln!("Invalid mode: {} (expected 'server' or 'client')", args[1]);
            std::process::exit(1);
        }
    }
}

fn run_server() -> std::io::Result<()> {
    // Find the hostname.
    let hostname = hostname::get()?;
    let hostname = hostname
        .into_string()
        .unwrap_or_else(|os| os.to_string_lossy().into_owned());

    // Bind to all interfaces on the chosen port.
    let socket = UdpSocket::bind(("0.0.0.0", PORT))?;

    println!("[server] Listening on UDP *:{}", PORT);

    let mut buf = [0u8; 512];

    loop {
        let (len, src) = socket.recv_from(&mut buf)?;

        if &buf[..len] == PING_MSG {
            socket.send_to(hostname.as_bytes(), src)?;
            println!("[server] Responded to {} with '{}'", src, hostname);
        }
    }
}

fn run_client() -> std::io::Result<()> {
    const TIMEOUT: Duration = Duration::from_secs(5);

    let socket = UdpSocket::bind(("0.0.0.0", 0))?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(TIMEOUT))?;

    socket.send_to(PING_MSG, ("255.255.255.255", PORT))?;
    println!("[client] Broadcast PING, awaiting replies...");

    let now = std::time::SystemTime::now();

    let mut buf = [0u8; 512];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                let reply = &buf[..len];
                println!("[client] {} → {}", src, String::from_utf8_lossy(reply));
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                let elapsed = now.elapsed().expect("elapsed");

                if elapsed > TIMEOUT {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                break;
            }
            Err(e) => return Err(e),
        }
    }

    println!("[client] Done.");

    Ok(())
}
