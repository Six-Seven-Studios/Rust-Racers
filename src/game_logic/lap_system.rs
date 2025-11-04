use crate::game_logic::{Car, PlayerControlled, AIControlled};
use crate::multiplayer::NetworkPlayer;
use crate::networking_plugin::NetworkClient;
use crate::GameState;
use bevy::prelude::*;

#[derive(Component)]
pub struct LapCounter {
    pub current_lap: u8,
    pub total_laps: u8,
    pub has_finished: bool,
    pub next_checkpoint: usize, // index of the next checkpoint to hit
}

impl Default for LapCounter {
    fn default() -> Self {
        Self {
            current_lap: 0,
            total_laps: 2, // two for now 
            has_finished: false,
            next_checkpoint: 0, 
        }
    }
}

#[derive(Component)]
pub struct FinishLine;

#[derive(Component)]
pub struct Checkpoint{
    pub index: usize, // order of checkpoints
}

pub fn spawn_lap_triggers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {

    let finish_line_handle = asset_server.load("finish_line.png");
    commands.spawn((
        FinishLine,
        Sprite::from_image(finish_line_handle),
        Transform {
            translation: Vec3::new(2752., 960., 10.),
            ..default()
        },
    ));

    // spawning checkpoints via a list
    let checkpoint_handle = asset_server.load("checkpoint.png");
    let checkpoint_positions = vec![
        // first check
        Vec3::new(1200., -2800., 10.),
        Vec3::new(2752., -2400., 10.),
        Vec3::new(2752., -1600., 10.),
        Vec3::new(2100., -800., 10.),
    ];

    for (i, pos) in checkpoint_positions.iter().enumerate() {
        commands.spawn((
            Checkpoint { index: i },
            Sprite::from_image(checkpoint_handle.clone()),
            Transform {
                translation: *pos,
                ..default()
            },
        ));
    }
    

}

pub fn update_laps(
    mut query_cars: Query<(
        Entity,
        &Transform,
        &mut LapCounter,
        Option<&PlayerControlled>,
        Option<&AIControlled>,
        Option<&NetworkPlayer>,
    ), With<Car>>,
    query_finish: Query<&Transform, With<FinishLine>>,
    query_checkpoints: Query<(&Transform, &Checkpoint)>,
    network_client: Res<NetworkClient>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(finish_transform) = query_finish.get_single() else {
        return;
    };

    // config
    let checkpoint_half_size = Vec2::new(860.0 / 2.0, 720.0 / 2.0);
    let finish_half_size = Vec2::new(860.0 / 2.0, 720.0 / 2.0);
    let padding = 0.0; // allows some buffer for detection

    let mut checkpoint_data: Vec<(Vec3, usize)> = query_checkpoints
        .iter()
        .map(|(t, c)| (t.translation, c.index))
        .collect();
    
    // sort to ensure 0, 1, 2, 3
    checkpoint_data.sort_by_key(|(_, i)| *i);

    for (entity, car_transform, mut lap_counter, player, ai, network) in query_cars.iter_mut() {
        // skip cars that have already finished the race
        if lap_counter.has_finished {
            continue;
        }

        // identify the car for logging
        let car_label = if player.is_some() {
            if let Some(my_id) = network_client.player_id {
                format!("Player {}", my_id)
            } else {
                "Player 1".to_string()
            }
        } else if let Some(net) = network {
            format!("Player {}", net.player_id)
        } else if ai.is_some() {
            format!("AI Car {}", entity.index())
        } else {
            format!("Unknown {}", entity.index())
        };

        let car_pos = car_transform.translation.truncate();

        // check next checkpoint
        if let Some((checkpoint_pos, index)) =
            checkpoint_data.iter().find(|(_, i)| *i == lap_counter.next_checkpoint)
        {
            let delta = car_pos - checkpoint_pos.truncate();

            if delta.x.abs() < checkpoint_half_size.x + padding
                && delta.y.abs() < checkpoint_half_size.y + padding
            {
                println!("{}: Reached checkpoint {}", car_label, index);
                lap_counter.next_checkpoint += 1;
                continue;
            }
            // debug
            //println!("car: ({:.0}, {:.0})  chk: ({:.0}, {:.0})  delta: ({:.0}, {:.0})",
            //car_pos.x, car_pos.y, checkpoint_pos.x, checkpoint_pos.y, delta.x, delta.y);
        }

        // check finish line
        if lap_counter.next_checkpoint == checkpoint_data.len() {
            let delta_finish = car_pos - finish_transform.translation.truncate();

            if delta_finish.x.abs() < finish_half_size.x + padding
                && delta_finish.y.abs() < finish_half_size.y + padding
            {
                lap_counter.current_lap += 1;
                lap_counter.next_checkpoint = 0;

                println!("{}: Lap complete {}", car_label, lap_counter.current_lap);

                if lap_counter.current_lap >= lap_counter.total_laps {
                    lap_counter.has_finished = true;
                    println!("{}: Finished all laps!", car_label);
                    next_state.set(GameState::Victory);
                }
            }
        }

    }
}