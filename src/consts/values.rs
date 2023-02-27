use sdl2::pixels::Color;

// Screen dimensions
pub const SCREEN_WIDTH: u32 = 1275;
pub const SCREEN_HEIGHT: u32 = 720;

// A scalar for how aggressive the camera zoom is
pub const CAMERA_ZOOM_INCREMENT: f32 = 0.015;
// Minimal zoom scale; Everything will be MIN_ZOOM_SCALE times smaller
pub const MIN_ZOOM_SCALE: u32 = 4;
// Maximal zoom scale; Everything will be 1 / MAX_ZOOM_SCALE times bigger
pub const MAX_ZOOM_SCALE: f32 = MIN_ZOOM_SCALE as f32 / 2.0;

// Map dimensions, calculated with screen dimensions and minimal zoom scale
pub const MAP_WIDTH: u32 = SCREEN_WIDTH * MIN_ZOOM_SCALE;
pub const MAP_HEIGHT: u32 = SCREEN_HEIGHT * MIN_ZOOM_SCALE;

// Small amount of render padding to the render area so that the screen never shows unredered pixels
pub const MAP_PADDING: u32 = 100;

// Entity health bar dimensions
pub const HEALTH_BAR_WIDTH: f32 = 100.0;
pub const HEALTH_BAR_HEIGHT: f32 = 8.0;
// How far above an entity it's health bar will sit (might wanna automate this later based on zoom scale?)
pub const HEALTH_BAR_Y_FLOAT: f32 = 35.0;

// Base unit stats
pub const BASE_UNIT_SPEED: f32 = 150.0; // How fast it can move
pub const BASE_UNIT_DAMAGE: f32 = 3.0; // How much damage it deals when attacking
pub const BASE_UNIT_RANGE: f32 = 125.0; // How far away can it attack
pub const ATTACKER_SPEED_PENALTY: f32 = 0.35; // A scalar that gets applied to unit speed while it is attacking

// Defines how much of an ent's dimensions gets extended into its selection box render
pub const SELECTION_BORDER_RATIO: f32 = 0.3;
// Defines a minimum size for selection borders
pub const MIN_SELECTION_BORDER_SIZE: f32 = 6.0;
// Defines a maximum size for selection borders
pub const MAX_SELECTION_BORDER_SIZE: f32 = 10.0;

// Defines the amount of time that should elapse between each physics step.
pub const TIME_STEP: f32 = 1.0 / 60.0;

// Color to clear the screen with
pub const SCREEN_BACKGROUND_COLOR: Color = Color::RGB(64, 192, 255);

// Color of the selection box, with alpha
pub const SELECTION_BOX_COLOR: Color = Color::RGBA(50, 150, 25, 100);
// Color of the entity selection border, with alpha
pub const SELECTION_BORDER_COLOR: Color = Color::RGBA(150, 0, 0, 225);
// Color of the entity selection target border, with alpha
pub const SELECTION_TARGET_BORDER_COLOR: Color = Color::RGBA(50, 225, 50, 225);

// Some useful color definitions
pub const RED_RGB: Color = Color::RGB(255, 0, 0);
pub const GREEN_RGB: Color = Color::RGB(0, 255, 0);
pub const BLACK_RGB: Color = Color::RGB(0, 0, 0);
