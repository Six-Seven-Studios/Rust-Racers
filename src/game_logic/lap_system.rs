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

pub fn spawn_lap_triggers(mut commands: Commands, asset_server: Res<AssetServer>) {
    let finish_line_handle = asset_server.load("finish_line.png");
    commands.spawn((
        FinishLine,
        Sprite::from_image(finish_line_handle),
        Transform {
            translation: Vec3::new(2752., 960., 5.),
            ..default()
        },
    ));

    // spawning checkpoints via a list
    // let checkpoint_handle = asset_server.load("twoBarrels.png");
    // let checkpoint_positions = vec![
    //     // first check
    //     Vec3::new(2752., 1500., 10.),

    //     Vec3::new(2752., 2800., 10.),

    //     Vec3::new(400., 2800., 10.),

    //     Vec3::new(-1600., 400., 10.),

    //     Vec3::new(-2044., -1493., 10.),

    //     Vec3::new(-1979., -2794., 10.),

    //     Vec3::new(1515., -2736., 10.),

    //     Vec3::new(2099., -150., 10.),
    // ];

    // for (i, pos) in checkpoint_positions.iter().enumerate() {
    //     commands.spawn((
    //         Checkpoint { index: i },
    //         Sprite::from_image(checkpoint_handle.clone()),
    //         Transform {
    //             translation: *pos,
    //             ..default()
    //         },
    //     ));
    // }

    let checkpoint_handle = asset_server.load("twoBarrels.png");
    let checkpoint_data = vec![
        // (position, rotation in radians)
        (Vec3::new(2752., 1500., 10.), 0.0),
        (Vec3::new(2700., 2700., 10.), std::f32::consts::PI / 4.0),
        (Vec3::new(425., 2725., 10.), std::f32::consts::PI / -4.0),
        (Vec3::new(-1600., 400., 10.), std::f32::consts::PI / -4.0),
        (Vec3::new(-2044., -1493., 10.), 0.0),
        (Vec3::new(-1979., -2750., 10.), std::f32::consts::PI / 2.0),
        (Vec3::new(1515., -2750., 10.), std::f32::consts::PI / 2.0),
        (Vec3::new(2100., -150., 10.), 0.0),
    ];

    for (i, (pos, rotation)) in checkpoint_data.iter().enumerate() {
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

    for (car_transform, mut lap_counter, player_flag) in query_cars.iter_mut() {
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
            // debug
            /*
            if player_flag.is_some() {
                info!(
                    "PLAYER car: ({:.0}, {:.0})  chk: ({:.0}, {:.0})  delta: ({:.0}, {:.0})",
                    car_pos.x, car_pos.y,
                    checkpoint_pos.x, checkpoint_pos.y,
                    delta.x, delta.y
                );
            }
            */
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
