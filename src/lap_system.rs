use crate::car::Car;
use crate::GameState;
use bevy::prelude::*;

#[derive(Component)]
pub struct LapCounter {
    pub current_lap: u8,
    pub total_laps: u8,
    pub has_finished: bool,
    pub reached_checkpoint: bool,
}

impl Default for LapCounter {
    fn default() -> Self {
        Self {
            current_lap: 0,
            total_laps: 3, 
            has_finished: false,
            reached_checkpoint: false,
        }
    }
}

#[derive(Component)]
pub struct FinishLine;

#[derive(Component)]
pub struct Checkpoint;

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

    let checkpoint_handle = asset_server.load("checkpoint.png");
    commands.spawn((
        Checkpoint,
        Sprite::from_image(checkpoint_handle),
        Transform::from_translation(Vec3::new(2752., 1920., 10.)),
    ));

}

pub fn update_laps(
    mut query_cars: Query<(&Transform, &mut LapCounter), With<Car>>,
    query_finish: Query<&Transform, With<FinishLine>>,
    query_checkpoint: Query<&Transform, With<Checkpoint>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let (Ok(finish_transform), Ok(checkpoint_transform)) = (
        query_finish.single(),
        query_checkpoint.single(),
    ) {
        for (car_transform, mut lap_counter) in query_cars.iter_mut() {
            let car_pos = car_transform.translation;

            let checkpoint_dist = car_pos.distance(checkpoint_transform.translation);
            let finish_dist = car_pos.distance(finish_transform.translation);

            // THE DIST IS RLY MESSED UP AND WEIRD. MIN DIST BETWEEN CAR AND FINISH LINE / CHECKPOINT SEEMS TO BE 890
            // this is why i hard coded dist to be less than 920
            if checkpoint_dist < 920.0 && !lap_counter.reached_checkpoint {
                lap_counter.reached_checkpoint = true;
                println!("CHECKPOINT REACHED");
            }

            if finish_dist < 920.0 && lap_counter.reached_checkpoint {
                lap_counter.current_lap += 1;
                lap_counter.reached_checkpoint = false;

                println!("LAP COMPLETE {}", lap_counter.current_lap);

                if lap_counter.current_lap >= lap_counter.total_laps {
                    lap_counter.has_finished = true;
                    println!("CAR FINISHED LAPS");
                    
                    next_state.set(GameState::Credits);
                }
            }
        }
    }
}
