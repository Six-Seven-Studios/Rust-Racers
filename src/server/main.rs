// Module declarations for shared code
#[path = "../game_logic/mod.rs"]
mod game_logic;

// Client modules needed because game_logic/lap_system imports them
#[path = "../car.rs"]
mod car;
#[path = "../car_state.rs"]
mod car_state;
#[path = "../multiplayer.rs"]
mod multiplayer;
#[path = "../networking_plugin.rs"]
mod networking_plugin;
#[path = "../networking.rs"]
mod networking;
#[path = "../lobby.rs"]
mod lobby;
#[path = "../title_screen.rs"]
mod title_screen;

// Server modules
mod types;
mod utils;
mod net;
mod lobby_management;
mod simulation;

use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use game_logic::{GameMap, load_map_from_file};
use types::*;
use utils::*;
use net::*;
use lobby_management::*;
use simulation::*;

fn main() {
    // Display the local IP address
    match get_local_ip() {
        Ok(ip) => println!("Server running on {}:4000", ip),
        Err(e) => println!("Server running on 0.0.0.0:4000 (Could not determine local IP: {})", e),
    }

    // Bind UDP socket
    let socket = UdpSocket::bind("0.0.0.0:4000").expect("Failed to bind UDP socket to port 4000");
    println!("UDP server listening on 0.0.0.0:4000");
    let socket = Arc::new(socket);

    // Set up shared resources for networking
    let connected_clients = ConnectedClients::new(Arc::clone(&socket));
    let lobbies: LobbyList = Arc::new(Mutex::new(Vec::new()));

    // Create command channel for networking threads to communicate with Bevy
    let (cmd_sender, cmd_receiver) = std::sync::mpsc::channel::<ServerCommand>();
    let cmd_sender = Arc::new(Mutex::new(cmd_sender));
    let cmd_receiver = Arc::new(Mutex::new(cmd_receiver));

    // Initialize Bevy's task pools
    bevy::tasks::IoTaskPool::get_or_init(|| bevy::tasks::TaskPool::new());

    // Clone for the listener thread
    let connected_clients_clone = ConnectedClients {
        ids: Arc::clone(&connected_clients.ids),
        addrs: Arc::clone(&connected_clients.addrs),
        addr_to_id: Arc::clone(&connected_clients.addr_to_id),
        last_seen: Arc::clone(&connected_clients.last_seen),
        socket: Arc::clone(&socket),
    };
    let lobbies_clone = Arc::clone(&lobbies);

    // Start the UDP listener in a separate thread
    server_listener(connected_clients_clone, lobbies_clone, Arc::clone(&cmd_sender));

    // Load the game map for server-side physics
    let game_map = load_map_from_file("assets/big-map.txt");
    println!("Server loaded map: {}x{}", game_map.width, game_map.height);

    // Create headless server
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
            Duration::from_millis(50),
        )))
        .insert_resource(connected_clients)
        .insert_resource(Lobbies { list: lobbies })
        .insert_resource(PlayerEntities::default())
        .insert_resource(ServerCommandReceiver { receiver: cmd_receiver })
        .insert_resource(game_map)
        .add_systems(Update, (
            process_server_commands_system,
            sync_input_from_lobbies_system,
            physics_simulation_system,
            timeout_cleanup_system,
        ).chain())
        .add_systems(FixedUpdate, broadcast_state_system)
        .run();
}
