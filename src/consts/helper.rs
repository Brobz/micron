use sdl2::{
    gfx::primitives::DrawRenderer,
    pixels::Color,
    rect::{Point, Rect},
    render::{BlendMode, Canvas},
    video::Window,
};
use vector2d::Vector2D;

use crate::{
    enums::{game_object::GameObject, unit_type::UnitType},
    structs::{
        ent::{Ent, EntID, EntParentType, Owner},
        order::{EntTarget, Order},
        unit::{Unit, UnitParentType},
        world::World,
        world_info::WorldInfo,
    },
};

use super::values::{
    BASE_COLLECTOR_MAX_HP, BASE_COLLECTOR_RECT_SIZE, BASE_MINER_MAX_HP, BASE_MINER_RECT_SIZE,
    BASE_SCOUT_MAX_HP, BASE_SCOUT_RECT_SIZE, COLLECTOR_ENT_COLOR, MINER_ENT_COLOR, SCOUT_ENT_COLOR,
    SELECTION_BORDER_SIZE,
};

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
    canvas.fill_rect(waypoint_rect).ok();
}

// Renders rect selection border behind selected entity
pub fn draw_rect_selection_border(canvas: &mut Canvas<Window>, ent_rect: &Rect, color: Color) {
    canvas.set_blend_mode(BlendMode::Blend);
    canvas.set_draw_color(color);
    let selection_border_rect: Rect = Rect::new(
        (ent_rect.x as f32 - (SELECTION_BORDER_SIZE / 2.0)) as i32,
        (ent_rect.y as f32 - (SELECTION_BORDER_SIZE / 2.0)) as i32,
        ent_rect.width() + SELECTION_BORDER_SIZE as u32,
        ent_rect.height() + SELECTION_BORDER_SIZE as u32,
    );
    canvas.fill_rect(selection_border_rect).ok();
    canvas.set_blend_mode(BlendMode::None);
}

// Renders circle selection border behind selected entity
pub fn draw_circle_selection_border(
    canvas: &mut Canvas<Window>,
    ent_position: Vector2D<f32>,
    ent_radius: i16,
    color: Color,
) {
    canvas.set_blend_mode(BlendMode::Blend);
    canvas.set_draw_color(color);
    canvas
        .filled_circle(
            ent_position.x as i16,
            ent_position.y as i16,
            ent_radius + 3 as i16,
            color,
        )
        .ok();
    canvas.set_blend_mode(BlendMode::None);
}

// Selects all (if any) player owned army units
// Note: replaces current selection)
pub fn select_all_army(world: &mut World) {
    for game_object in &mut world.game_objects {
        match game_object {
            GameObject::Unit(ent, unit_type) => {
                if ent.owner == Owner::Player {
                    match unit_type {
                        UnitType::Scout(_) => ent.select(),
                        UnitType::Miner(_) => ent.deselect(),
                        UnitType::Collector(_) => ent.deselect(),
                    }
                } else {
                    ent.deselect();
                }
            }
            _ => {}
        }
    }
    world.selection.open = false;
}

// Returns an emtpy EntTarget object
pub fn empty_ent_target() -> EntTarget {
    EntTarget {
        ent_id: None,
        ent_rect: None,
        ent_owner: None,
        ent_parent_type: None,
    }
}

// Returns a new GameObject with the appropriate unit stats
pub fn new_unit(
    world_info: &mut WorldInfo,
    unit_type: UnitParentType,
    owner: Owner,
    position: Vector2D<f32>,
) -> GameObject {
    match unit_type {
        UnitParentType::Miner => {
            let new_ent = Ent::new(
                EntParentType::Unit,
                owner,
                BASE_MINER_MAX_HP,
                position,
                Point::new(BASE_MINER_RECT_SIZE, BASE_MINER_RECT_SIZE),
                MINER_ENT_COLOR,
            );
            world_info.add_ent(&new_ent);
            GameObject::Unit(new_ent, UnitType::Miner(Unit::new(unit_type)))
        }
        UnitParentType::Scout => {
            let new_ent = Ent::new(
                EntParentType::Unit,
                owner,
                BASE_SCOUT_MAX_HP,
                position,
                Point::new(BASE_SCOUT_RECT_SIZE, BASE_SCOUT_RECT_SIZE),
                SCOUT_ENT_COLOR,
            );
            world_info.add_ent(&new_ent);
            GameObject::Unit(new_ent, UnitType::Scout(Unit::new(unit_type)))
        }
        UnitParentType::Collector => {
            let new_ent = Ent::new(
                EntParentType::Unit,
                owner,
                BASE_COLLECTOR_MAX_HP,
                position,
                Point::new(BASE_COLLECTOR_RECT_SIZE, BASE_COLLECTOR_RECT_SIZE),
                COLLECTOR_ENT_COLOR,
            );
            world_info.add_ent(&new_ent);
            GameObject::Unit(new_ent, UnitType::Collector(Unit::new(unit_type)))
        }
    }
}
