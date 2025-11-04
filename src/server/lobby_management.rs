use serde_json::json;
use std::time::Instant;

use crate::types::*;

/// Broadcast the current lobby state to all players in the lobby
pub fn broadcast_lobby_state(
    connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
    lobby_index: usize,
) {
    let guard = lobbies.lock().unwrap();

    let lobby = if let Some(i) = guard.get(lobby_index) {
        i
    } else {
        println!("Lobby does not exist");
        return;
    };

    // Snapshot the player IDs
    let players: Vec<u32> = {
        let lobby_guard = lobby.players.lock().unwrap();
        lobby_guard.clone()
    };

    // Build one payload that everyone in this lobby gets
    let payload = json!({
        "lobby": lobby.name.clone(),
        "players": players
    }).to_string() + "\n";

    // Get target addresses and send
    let addrs = connected_clients.addrs.lock().unwrap();
    for pid in &players {
        if let Some(addr) = addrs.get(pid) {
            let _ = connected_clients.socket.send_to(payload.as_bytes(), addr);
        }
    }
}

/// Broadcast the list of active lobbies to all connected clients
pub fn broadcast_active_lobbies(
    connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
) {
    let guard = lobbies.lock().unwrap();

    let lobby_list: Vec<_> = guard.iter().map(|lobby| {
        let players = lobby.players.lock().unwrap();
        json!({
            "name": lobby.name.clone(),
            "players": players.len()
        })
    }).collect();

    let payload = json!({
        "type": "active_lobbies",
        "lobbies": lobby_list
    }).to_string() + "\n";

    // Send to all connected clients
    let addrs = connected_clients.addrs.lock().unwrap();
    for addr in addrs.values() {
        let _ = connected_clients.socket.send_to(payload.as_bytes(), addr);
    }
}

/// Broadcast game start message to all players in a lobby
pub fn broadcast_game_start(
    connected_clients: &ConnectedClients,
    players: &[u32],
    lobby_name: &str,
) {
    let payload = json!({
        "type": "game_started",
        "lobby": lobby_name
    }).to_string() + "\n";

    // Send to all players in the lobby
    let addrs = connected_clients.addrs.lock().unwrap();
    for pid in players {
        if let Some(addr) = addrs.get(pid) {
            let _ = connected_clients.socket.send_to(payload.as_bytes(), addr);
        }
    }
}

/// Clean up when a client disconnects
pub fn disconnect_cleanup(id: u32, connected: &ConnectedClients, lobbies: &LobbyList) {
    // Get the address before removing
    let addr = connected.addrs.lock().unwrap().get(&id).copied();

    // Remove from all maps
    if let Ok(mut m) = connected.addrs.lock() { m.remove(&id); }
    if let Ok(mut ids) = connected.ids.lock() { ids.retain(|x| *x != id); }
    if let Ok(mut last_seen) = connected.last_seen.lock() { last_seen.remove(&id); }
    if let Some(addr) = addr {
        if let Ok(mut addr_to_id) = connected.addr_to_id.lock() {
            addr_to_id.remove(&addr);
        }
    }

    let mut empty_lobbies: Vec<usize> = Vec::new();

    // Remove from all lobbies
    let mut guard = lobbies.lock().unwrap();
    for (i, lobby) in guard.iter_mut().enumerate() {
        let mut players = lobby.players.lock().unwrap();
        players.retain(|&pid| pid != id);

        if players.is_empty() {
            empty_lobbies.push(i);
        } else if lobby.host == id {
            if let Some(first_player) = players.get(0) {
                lobby.host = *first_player;
            } else {
                println!("Should never get here because players should have at least one element");
            }
        }
    }

    // Remove the empty lobbies
    for index in empty_lobbies.iter().rev() {
        guard.remove(*index);
    }

    println!("Client {} disconnected and cleaned up", id);
}

/// Check for timed out clients and disconnect them
pub fn timeout_cleanup(
    connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
    timeout_seconds: u64,
) {
    let now = Instant::now();
    let mut timed_out_clients = Vec::new();

    // Find clients that haven't sent anything in timeout_seconds
    {
        let last_seen = connected_clients.last_seen.lock().unwrap();
        for (id, last_time) in last_seen.iter() {
            if now.duration_since(*last_time).as_secs() > timeout_seconds {
                timed_out_clients.push(*id);
            }
        }
    }

    // Disconnect timed out clients
    for id in timed_out_clients {
        println!("Client {} timed out", id);
        disconnect_cleanup(id, connected_clients, lobbies);
    }
}
