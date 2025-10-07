use bevy::prelude::*;
use std::io::Write;
use std::net::TcpListener;
use std::thread;

/// Plugin that starts a background TCP listener
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, server_listener);
    }
}

fn server_listener() {
    // Run the TCP listener in a background thread so Bevy can render normally
    thread::spawn(move || {
        let listener = TcpListener::bind(("0.0.0.0", 4000)).expect("Expected to bind to port 4000 successfully");
        println!("Listening on 0.0.0.0:4000");
        let mut next_id: u32 = 1;

        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    let id = next_id;
                    next_id += 1;
                    let line = format!("WELCOME PLAYER {}\n", id);
                    let _ = s.write_all(line.as_bytes());
                    let _ = s.flush();
                    println!("Greeted client with id={id}");
                }
                Err(e) => eprintln!("Accept error: {e}"),
            }
        }
    });
}