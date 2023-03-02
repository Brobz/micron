use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::{BlendMode, Canvas},
    video::Window,
};
use vector2d::Vector2D;

use crate::{
    enums::game_object::GameObject,
    structs::{
        ent::{EntID, Owner},
        order::{EntTarget, Order},
        world::World,
    },
};

use super::values::SELECTION_BORDER_SIZE;

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
pub fn draw_waypoint(order: Order, canvas: &mut Canvas<Window>) {
    let waypoint_rect: Rect = Rect::from_center(
        Point::new(
            order.current_move_target.x as i32,
            order.current_move_target.y as i32,
        ),
        5,
        5,
    );
    canvas.fill_rect(waypoint_rect).ok().unwrap_or_default();
}

// Renders selection border behind selected entities
// Selection border will be calculated using the selection border ration and the entity rect dimensions
// It also gets clamped to ensure it doesn't look off on entities that are too big or too smal
pub fn draw_selection_border(canvas: &mut Canvas<Window>, ent_rect: &Rect, color: Color) {
    canvas.set_blend_mode(BlendMode::Blend);
    canvas.set_draw_color(color);
    let selection_border_rect: Rect = Rect::new(
        (ent_rect.x as f32 - (SELECTION_BORDER_SIZE / 2.0)) as i32,
        (ent_rect.y as f32 - (SELECTION_BORDER_SIZE / 2.0)) as i32,
        ent_rect.width() + SELECTION_BORDER_SIZE as u32,
        ent_rect.height() + SELECTION_BORDER_SIZE as u32,
    );
    canvas
        .fill_rect(selection_border_rect)
        .ok()
        .unwrap_or_default();
    canvas.set_blend_mode(BlendMode::None);
}

// Selects all (if any) player owned army units
// Note: replaces current selection)
pub fn select_all_army(world: &mut World) {
    for game_object in &mut world.game_objects {
        match game_object {
            GameObject::Unit(ent, _) => {
                if ent.owner == Owner::Player {
                    // TODO: FOR NOW, all units are army; soon this will change
                    //      if unit.type is army type or whatever
                    ent.select();
                } else {
                    ent.deselect();
                }
            }
            _ => {}
        }
    }
    world.selection.open = false;
}

pub fn empty_ent_target() -> EntTarget {
    EntTarget {
        ent_id: None,
        ent_rect: None,
        ent_owner: None,
        ent_parent_type: None,
    }
}
