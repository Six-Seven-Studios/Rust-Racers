use std::io::{self, BufRead, BufReader};
use std::net::TcpStream;

fn main() {
    println!("Enter server IP:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let addr = format!("{}:4000", input.trim());
    
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