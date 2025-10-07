use std::io::{BufRead, BufReader};
use std::net::TcpStream;

fn main() {
    let addr = "127.0.0.1:4000";
    println!("Connecting to {addr}...");
    match TcpStream::connect(addr) {
        Ok(stream) => {
            println!("Connected to server!");
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            if let Ok(n) = reader.read_line(&mut line) {
                if n > 0 {
                    println!("Received: {}", line.trim_end());
                } else {
                    println!("Server closed connection with no data");
                }
            } else {
                println!("Failed to read greeting");
            }
        }
        Err(e) => eprintln!("Connect error: {e}"),
    }
}