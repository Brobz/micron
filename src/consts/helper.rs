use sdl2::{
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};
use vector2d::Vector2D;

use crate::structs::{ent::EntID, order::Order};

// Counter to guarantee a unique EntID
pub static mut CURRENT_ENT_ID: EntID = EntID(0);

// This method returns a normalized vector with size speed that points from, to
pub fn get_direction_from_to(from: Vector2D<f32>, to: Vector2D<f32>, speed: f32) -> Vector2D<f32> {
    (to - from).normalise() * speed
}

// This method takes in a vec2 position of where the mouse is currently, and one of where it was originally clicked;
// Then it returns a vec2 which pertains to the left corner of a square where each one of the two inputs represent opposing corners
pub fn find_selection_box_translation(curr_pos: Point, origin: Point) -> Point {
    Point::new(
        if origin.x > curr_pos.x {
            curr_pos.x
        } else {
            origin.x
        },
        if origin.y > curr_pos.y {
            curr_pos.y
        } else {
            origin.y
        },
    )
}

// This method renders an order waypoint to the screen
pub fn draw_waypoint(order: &Order, canvas: &mut Canvas<Window>) {
    let waypoint_rect: Rect = Rect::from_center(
        Point::new(order.move_target.x as i32, order.move_target.y as i32),
        5,
        5,
    );
    canvas.fill_rect(waypoint_rect).ok().unwrap_or_default();
}
