use bevy::input::ButtonInput;
use bevy::prelude::*;
use crate::GameState;

pub fn check_for_title_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if input.just_pressed(KeyCode::Space) && *current_state == GameState::Title {
        next_state.set(GameState::Playing);
    }
}