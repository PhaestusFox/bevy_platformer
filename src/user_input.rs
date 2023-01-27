use bevy::prelude::KeyCode;
use leafwing_input_manager::prelude::*;


#[derive(Debug, Actionlike, Clone)]
pub enum PlayerInput {
    Left,
    Right,
    Jump,
    Fall,
}

impl PlayerInput {
    pub fn player_one() -> InputMap<PlayerInput> {
        let mut map = InputMap::default();
        map.insert_multiple([
            (KeyCode::A, PlayerInput::Left),
            (KeyCode::Left, PlayerInput::Left),
            (KeyCode::D, PlayerInput::Right),
            (KeyCode::Right, PlayerInput::Right),
            (KeyCode::W, PlayerInput::Jump),
            (KeyCode::Space, PlayerInput::Jump),
            (KeyCode::Up, PlayerInput::Jump),
            (KeyCode::S, PlayerInput::Fall),
            (KeyCode::Down, PlayerInput::Fall),
            ]);
        map
    }
}