use sdl2::{
    mouse::MouseWheelDirection,
    rect::{Point, Rect},
};
use vector2d::Vector2D;

use crate::consts::values::{
    CAMERA_ZOOM_INCREMENT, MAP_HEIGHT, MAP_WIDTH, MAX_ZOOM_SCALE, MIN_ZOOM_SCALE, SCREEN_HEIGHT,
    SCREEN_WIDTH,
};

pub struct Camera {
    pub position: Vector2D<i32>,
    pub scale: Vector2D<f32>,
    pub mouse_rect: Rect,
    is_anchored: bool,
    anchor_position: Option<Vector2D<i32>>,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vector2D::<i32>::new(MAP_WIDTH as i32 / -2, MAP_HEIGHT as i32 / -2),
            scale: Vector2D::<f32>::new(1.0, 1.0),
            mouse_rect: Rect::new(-1, -1, 2, 2),
            is_anchored: false,
            anchor_position: None,
        }
    }

    pub fn update_mouse_rect(&mut self, mouse_position: Point) {
        self.mouse_rect.x = mouse_position.x;
        self.mouse_rect.y = mouse_position.y;
    }

    pub fn drag_to(&mut self, position: Point) {
        if let Some(anchor_position) = self.anchor_position {
            self.position.y = position.y - anchor_position.y;
            self.position.x = position.x - anchor_position.x;
        }
        self.clamp_camera_to_map_bounds();
    }
    pub fn is_anchored(&self) -> bool {
        self.is_anchored
    }

    pub fn grab(&mut self, position: &Point) {
        self.is_anchored = true;
        self.anchor_position = Some(Vector2D::<i32>::new(position.x, position.y));
    }

    pub fn release(&mut self) {
        self.is_anchored = false;
    }

    pub fn get_scaled_screen_area(&self) -> Rect {
        Rect::new(
            0,
            0,
            (MAP_WIDTH as f32 / self.scale.x) as u32,
            (MAP_HEIGHT as f32 / self.scale.y) as u32,
        )
    }

    pub fn clamp_camera_to_map_bounds(&mut self) {
        // Clamping position first ensures that the player will always be able to zoom in / out untill MAX / MIN ZOOM_SCALE, independent of camera position
        // (camera will move to accomodate zoom)
        self.clam_position_to_map_bounds();
        self.clamp_scale_to_map_bounds();
    }

    pub fn clam_position_to_map_bounds(&mut self) {
        if (-self.position.x as f32 + SCREEN_WIDTH as f32 / self.scale.x) > MAP_WIDTH as f32 {
            self.position.x = -(MAP_WIDTH as i32) + (SCREEN_WIDTH as f32 / self.scale.x) as i32;
        }
        if (-self.position.y as f32 + SCREEN_HEIGHT as f32 / self.scale.y) > MAP_HEIGHT as f32 {
            self.position.y = -(MAP_HEIGHT as i32) + (SCREEN_HEIGHT as f32 / self.scale.y) as i32;
        }

        if self.position.x > 0 {
            self.position.x = 0;
        }
        if self.position.y > 0 {
            self.position.y = 0;
        }
    }

    pub fn clamp_scale_to_map_bounds(&mut self) {
        if (-self.position.x as f32 + SCREEN_WIDTH as f32 / self.scale.x) > MAP_WIDTH as f32 {
            self.scale.x = SCREEN_WIDTH as f32 / (MAP_WIDTH as f32 + self.position.x as f32);
        }
        if (-self.position.y as f32 + SCREEN_HEIGHT as f32 / self.scale.y) > MAP_HEIGHT as f32 {
            self.scale.y = SCREEN_HEIGHT as f32 / (MAP_HEIGHT as f32 + self.position.y as f32);
        }

        if self.scale.y > MAX_ZOOM_SCALE {
            self.scale.y = MAX_ZOOM_SCALE;
        }
        if self.scale.x > MAX_ZOOM_SCALE {
            self.scale.x = MAX_ZOOM_SCALE;
        }
    }

    pub fn get_anchored_mouse_pos(&self) -> Point {
        Point::new(
            (self.mouse_rect.x as f32 / self.scale.x) as i32,
            (self.mouse_rect.y as f32 / self.scale.y) as i32,
        )
    }

    pub fn get_scaled_mouse_pos(&self) -> Point {
        Point::new(
            ((self.mouse_rect.x - (self.position.x as f32 * self.scale.x) as i32) as f32
                / self.scale.x) as i32,
            ((self.mouse_rect.y - (self.position.y as f32 * self.scale.y) as i32) as f32
                / self.scale.y) as i32,
        )
    }

    pub fn get_scaled_mouse_rect(&self) -> Rect {
        let scaled_mouse_pos = self.get_scaled_mouse_pos();
        Rect::new(
            scaled_mouse_pos.x,
            scaled_mouse_pos.y,
            self.mouse_rect.width(),
            self.mouse_rect.height(),
        )
    }

    pub fn zoom(&mut self, direction: MouseWheelDirection, amount: i32) {
        let zoom_amount = CAMERA_ZOOM_INCREMENT * amount as f32;

        // If we have a zoom amount or 0, or are out of bounds to zoom further / less, return early
        match zoom_amount {
            _ if zoom_amount == 0.0 => return {},
            _ if zoom_amount > 0.0 && self.scale.x == MAX_ZOOM_SCALE => return {},
            _ if zoom_amount < 0.0 && self.scale.x == MIN_ZOOM_SCALE as f32 => return {},
            _ => (),
        }

        let zoom_delta = Vector2D::<f32>::new(zoom_amount, zoom_amount);

        // Calculate current map size
        let current_map_size_vector = Vector2D {
            x: SCREEN_WIDTH as f32 / self.scale.x,
            y: SCREEN_HEIGHT as f32 / self.scale.y,
        };

        // Apply zoom delta to scale
        match direction {
            sdl2::mouse::MouseWheelDirection::Normal => {
                self.scale += zoom_delta.mul_components(self.scale);
            }
            sdl2::mouse::MouseWheelDirection::Flipped => {
                self.scale -= zoom_delta.mul_components(self.scale);
            }
            sdl2::mouse::MouseWheelDirection::Unknown(_) => (),
        }

        // Calculate new map size
        let new_map_size_vector = Vector2D {
            x: SCREEN_WIDTH as f32 / self.scale.x,
            y: SCREEN_HEIGHT as f32 / self.scale.y,
        };

        // Get a copy of mouse position as a vector
        let mouse_pos_vector = Vector2D {
            x: self.mouse_rect.x as f32,
            y: self.mouse_rect.y as f32,
        };

        // Get a copy of screen dimensions as a vector
        let screen_size_vector = Vector2D {
            x: SCREEN_WIDTH as f32,
            y: SCREEN_HEIGHT as f32,
        };

        // Calculate camera position delta
        // In order for the camera to always zoom towards the mouse cursor,
        // we need to move the camera towards the current cursor position,
        // weighted by how much we zoom compared to the screen size
        // TODO: produces weird zoom out; figure out a nicer one please ; -- ;
        let position_delta = mouse_pos_vector
            .div_components(screen_size_vector)
            .mul_components(new_map_size_vector - current_map_size_vector);

        // Apply it
        self.position += Vector2D::<i32>::new(position_delta.x as i32, position_delta.y as i32);

        self.clamp_camera_to_map_bounds();
    }
}
