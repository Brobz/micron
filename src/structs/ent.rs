use sdl2::rect::{Point, Rect};
use vector2d::Vector2D;

use crate::consts::helper::CURRENT_ENT_ID;
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntID(pub u64);

pub struct Ent {
    pub id: EntID,
    pub position: Vector2D<f32>,
    pub rect_size: Point,
    pub radius: i32,
    pub max_hp: u32,
    pub hp: f32,
}

impl Ent {
    pub fn new(max_hp: u32, position: Vector2D<f32>, rect_size: Point) -> Ent {
        unsafe {
            CURRENT_ENT_ID.0 += 1;
        }
        Ent {
            id: unsafe { CURRENT_ENT_ID },
            position,
            rect_size,
            radius: -1,
            max_hp,
            hp: max_hp as f32,
        }
    }

    pub fn get_rect(&self) -> Rect {
        Rect::new(
            self.position.x as i32,
            self.position.y as i32,
            self.rect_size.x as u32,
            self.rect_size.y as u32,
        )
    }
}
