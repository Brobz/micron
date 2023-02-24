use sdl2::rect::Point;
use vector2d::Vector2D;

use crate::structs::ent::EntID;

pub static mut CURRENT_ENT_ID: EntID = EntID(0);

// This method returns a normalized vector with size speed that points from, to
pub fn get_direction_from_to(from: Vector2D<f32>, to: Vector2D<f32>, speed: f32) -> Vector2D<f32> {
    return (to - from).normalise() * speed;
}

// This method takes in a vec2 position of where the mouse is currently, and one of where it was originally clicked;
// Then it returns a vec2 which pertains to the left corner of a square where each one of the two inputs represent opposing corners
pub fn find_selection_box_translation(curr_pos: Point, origin: Point) -> Point {
    Point::new(
        if origin.x > curr_pos.x {
            curr_pos.x as i32
        } else {
            origin.x as i32
        },
        if origin.y > curr_pos.y {
            curr_pos.y as i32
        } else {
            origin.y as i32
        },
    )
}
