// Module declarations for shared code
#[path = "../game_logic/mod.rs"]
mod game_logic;

// Client modules needed because game_logic/lap_system imports them
#[path = "../car.rs"]
mod car;
#[path = "../car_skins.rs"]
mod car_skins;
#[path = "../car_state.rs"]
mod car_state;
#[path = "../drift_settings.rs"]
mod drift_settings;
#[path = "../interpolation.rs"]
mod interpolation;
#[path = "../lobby.rs"]
mod lobby;
#[path = "../multiplayer.rs"]
mod multiplayer;
#[path = "../networking.rs"]
mod networking;
#[path = "../networking_plugin.rs"]
mod networking_plugin;
#[path = "../speed.rs"]
mod speed;
#[path = "../title_screen.rs"]
mod title_screen;

// Server modules
mod client_prediction;
mod lobby_management;
mod net;
mod simulation;
mod types;
mod utils;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use game_logic::{GameMap, SERVER_TIMESTEP, load_map_from_file};
use lobby_management::*;
use net::*;
use simulation::*;
use types::*;
use utils::*;

use speed::{
    SpeedBoost, SpeedPowerup, collect_powerups, remove_boost_ui, spawn_boost_ui,
    spawn_speed_powerups, update_speed_boost,
};

fn main() {
    // Display the local IP address
    match get_local_ip() {
        Ok(ip) => println!("Server running on {}:4000", ip),
        Err(e) => println!(
            "Server running on 0.0.0.0:4000 (Could not determine local IP: {})",
            e
        ),
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
    server_listener(
        connected_clients_clone,
        lobbies_clone,
        Arc::clone(&cmd_sender),
    );

    // Create headless server with 20 Hz timestep
    // Using Update schedule since run_loop already controls the rate
    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f32(
                SERVER_TIMESTEP,
            ))),
        )
        .insert_resource(connected_clients)
        .insert_resource(Lobbies { list: lobbies })
        .insert_resource(PlayerEntities::default())
        .insert_resource(ServerCommandReceiver {
            receiver: cmd_receiver,
        })
        .insert_resource(ServerCommandSender { sender: cmd_sender })
        .add_systems(
            Update,
            (
                process_server_commands_system,
                sync_input_from_lobbies_system,
                physics_simulation_system,
                ai_movement_system,
                broadcast_state_system,
                timeout_cleanup_system,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                spawn_speed_powerups,
                collect_powerups,
                update_speed_boost,
                spawn_boost_ui,
                remove_boost_ui,
            )
                .run_if(in_state(GameState::PlayingDemo).or(in_state(GameState::Playing))),
        )
        .run();
}
