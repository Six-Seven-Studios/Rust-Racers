use std::net::UdpSocket;

/// Get the local IP address for display purposes
pub fn get_local_ip() -> Result<String, Box<dyn std::error::Error>> {
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}
