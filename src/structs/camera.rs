use sdl2::{
    mouse::MouseWheelDirection,
    rect::{Point, Rect},
};
use vector2d::Vector2D;

use crate::consts::setup::{
    CAMERA_ZOOM_INCREMENT, MAP_HEIGHT, MAP_WIDTH, MAX_ZOOM_SCALE, SCREEN_HEIGHT, SCREEN_WIDTH,
};

pub struct Camera {
    pub position: Vector2D<i32>,
    pub scale: Vector2D<f32>,
    is_anchored: bool,
    anchor_position: Option<Vector2D<i32>>,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vector2D::<i32>::new(MAP_WIDTH as i32 / -2, MAP_HEIGHT as i32 / -2),
            scale: Vector2D::<f32>::new(1.0, 1.0),
            is_anchored: false,
            anchor_position: None,
        }
    }
    pub fn drag_to(&mut self, position: Point) {
        if let Some(anchor_position) = self.anchor_position {
            self.position.y = position.y - anchor_position.y;
            self.position.x = position.x - anchor_position.x;
        }
        self.clamp_position_to_map_bounds();
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

    pub fn clamp_position_to_map_bounds(&mut self) {
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

        self.clamp_scale_to_map_bounds();
    }

    pub fn clamp_scale_to_map_bounds(&mut self) {
        if (-self.position.x as f32 + SCREEN_WIDTH as f32 / self.scale.x) > MAP_WIDTH as f32 {
            self.scale.x = SCREEN_WIDTH as f32 / (MAP_WIDTH as f32 + self.position.x as f32);
        }
        if (-self.position.y as f32 + SCREEN_HEIGHT as f32 / self.scale.y) > MAP_HEIGHT as f32 {
            self.scale.y = SCREEN_HEIGHT as f32 / (MAP_HEIGHT as f32 + self.position.y as f32);
        }

        if self.scale.y > MAX_ZOOM_SCALE as f32 {
            self.scale.y = MAX_ZOOM_SCALE as f32;
        }
        if self.scale.x > MAX_ZOOM_SCALE as f32 {
            self.scale.x = MAX_ZOOM_SCALE as f32;
        }
    }

    pub fn get_anchored_mouse_pos(&self, mouse_x: i32, mouse_y: i32) -> Point {
        Point::new(
            (mouse_x as f32 / self.scale.x) as i32,
            (mouse_y as f32 / self.scale.y) as i32,
        )
    }

    pub fn get_scaled_mouse_pos(&self, mouse_x: i32, mouse_y: i32) -> Point {
        Point::new(
            ((mouse_x - (self.position.x as f32 * self.scale.x) as i32) as f32 / self.scale.x)
                as i32,
            ((mouse_y - (self.position.y as f32 * self.scale.y) as i32) as f32 / self.scale.y)
                as i32,
        )
    }

    pub fn zoom(&mut self, direction: MouseWheelDirection, amount: i32) {
        let zoom_amount = CAMERA_ZOOM_INCREMENT * amount as f32;
        let zoom_delta = Vector2D::<f32>::new(zoom_amount, zoom_amount);
        match direction {
            sdl2::mouse::MouseWheelDirection::Normal => {
                self.scale += zoom_delta.mul_components(self.scale);
            }
            sdl2::mouse::MouseWheelDirection::Flipped => {
                self.scale -= zoom_delta.mul_components(self.scale);
            }
            sdl2::mouse::MouseWheelDirection::Unknown(_) => (),
        }
        self.clamp_position_to_map_bounds();
    }
}
