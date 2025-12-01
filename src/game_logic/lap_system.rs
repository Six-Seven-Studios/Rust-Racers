use crate::GameState;
use crate::game_logic::{AIControlled, Car, PlayerControlled};
use crate::multiplayer::NetworkPlayer;
use crate::networking_plugin::NetworkClient;
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
pub struct Checkpoint {
    pub index: usize, // order of checkpoints
}

// map level component
#[derive(Resource, Clone, Default)]
pub struct MapLevelData {
    pub start_position: Vec3, // Where the player spawns
    pub finish_line_pos: Vec3,
    pub checkpoints: Vec<(Vec3, f32)>, // Position, Rotation (radians)
}

pub fn spawn_lap_triggers(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    map_data: Res<MapLevelData>
) {
    let finish_line_handle = asset_server.load("finish_line.png");
    
    // Spawn Finish Line from Resource
    commands.spawn((
        FinishLine,
        Sprite::from_image(finish_line_handle),
        Transform {
            translation: map_data.finish_line_pos,
            ..default()
        },
    ));

    let checkpoint_handle = asset_server.load("twoBarrels.png");

    // Spawn Checkpoints from Resource
    for (i, (pos, rotation)) in map_data.checkpoints.iter().enumerate() {
        commands.spawn((
            Checkpoint { index: i },
            Sprite::from_image(checkpoint_handle.clone()),
            Transform {
                translation: *pos,
                rotation: Quat::from_rotation_z(*rotation),
                ..default()
            },
        ));
    }
}


pub fn update_laps(
    mut query_cars: Query<(&Transform, &mut LapCounter, Option<&PlayerControlled>), With<Car>>,
    query_finish: Query<&Transform, With<FinishLine>>,
    query_checkpoints: Query<(&Transform, &Checkpoint)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(finish_transform) = query_finish.single() else {
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

    for (car_transform, mut lap_counter, _player_flag) in query_cars.iter_mut() {
        let car_pos = car_transform.translation.truncate();

        // check next checkpoint
        if let Some((checkpoint_pos, index)) = checkpoint_data
            .iter()
            .find(|(_, i)| *i == lap_counter.next_checkpoint)
        {
            let delta = car_pos - checkpoint_pos.truncate();

            if delta.x.abs() < checkpoint_half_size.x + padding
                && delta.y.abs() < checkpoint_half_size.y + padding
            {
                info!("Reached checkpoint {}", index);
                lap_counter.next_checkpoint += 1;
            }
        }
        // check finish line
        if lap_counter.next_checkpoint >= checkpoint_data.len() {
            let delta_finish = car_pos - finish_transform.translation.truncate();

            if delta_finish.x.abs() < finish_half_size.x + padding
                && delta_finish.y.abs() < finish_half_size.y + padding
            {
                lap_counter.current_lap += 1;
                lap_counter.next_checkpoint = 0;

                info!("Lap complete {}", lap_counter.current_lap);

                if lap_counter.current_lap >= lap_counter.total_laps {
                    lap_counter.has_finished = true;
                    info!("Car finished all laps!");
                    next_state.set(GameState::Victory);
                }
            }
        }
    }
}