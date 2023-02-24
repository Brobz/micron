use sdl2::pixels::Color;

pub const SCREEN_WIDTH: u32 = 1275;
pub const SCREEN_HEIGHT: u32 = 720;

pub const BASE_UNIT_SPEED: f32 = 150.0;

// Defines the amount of time that should elapse between each physics step.
pub const TIME_STEP: f32 = 1.0 / 60.0;

pub const SELECTION_BOX_COLOR: Color = Color::RGBA(50, 150, 25, 100);
