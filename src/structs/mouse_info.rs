use bevy::prelude::*;

use crate::consts::setup::SCREEN_HEIGHT;
use crate::consts::setup::SCREEN_WIDTH;

// This resource tracks the mouse position and button states
#[derive(Resource)]
pub struct MouseInfo {
    pub position: Vec2,
    pub left_button: bool,
    pub right_button: bool,
}

impl MouseInfo {
    pub fn set_pos(&mut self, _pos: Vec2) {
        self.position = Vec2::from([_pos.x - SCREEN_WIDTH / 2f32, _pos.y - SCREEN_HEIGHT / 2f32])
    }
}
