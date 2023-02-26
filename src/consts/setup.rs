use sdl2::pixels::Color;

pub const SCREEN_WIDTH: u32 = 1275;
pub const SCREEN_HEIGHT: u32 = 720;

pub const CAMERA_ZOOM_INCREMENT: f32 = 0.015;
pub const MAX_ZOOM_SCALE: u32 = 4;

pub const MAP_WIDTH: u32 = SCREEN_WIDTH * MAX_ZOOM_SCALE;
pub const MAP_HEIGHT: u32 = SCREEN_HEIGHT * MAX_ZOOM_SCALE;

pub const MAP_PADDING: u32 = 100;

pub const HEALTH_BAR_WIDTH: f32 = 100.0;
pub const HEALTH_BAR_HEIGHT: f32 = 8.0;
pub const HEALTH_BAR_Y_FLOAT: f32 = 35.0;

pub const BASE_UNIT_SPEED: f32 = 150.0;
pub const BASE_UNIT_DAMAGE: f32 = 3.0;
pub const BASE_UNIT_RANGE: f32 = 125.0;
pub const ATTACKER_SPEED_PENALTY: f32 = 0.35;

// Defines the amount of time that should elapse between each physics step.
pub const TIME_STEP: f32 = 1.0 / 60.0;

pub const SELECTION_BOX_COLOR: Color = Color::RGBA(50, 150, 25, 100);
