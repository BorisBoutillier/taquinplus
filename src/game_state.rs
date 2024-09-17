use crate::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Setup,
    Menu,
    PuzzleSolving,
    PuzzleSolved,
}

pub fn transition_setup_to_menu(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Menu);
}
