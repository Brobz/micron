use sdl2::rect::{Point, Rect};
use vector2d::Vector2D;

pub struct Ent {
    pub position: Vector2D<f32>,
    pub rect_size: Point,
    pub radius: i32,
    pub max_hp: f32,
    pub hp: f32,
}

impl Ent {
    pub fn new(max_hp: f32, position: Vector2D<f32>, rect_size: Point) -> Ent {
        Ent {
            position,
            rect_size,
            radius: -1,
            max_hp,
            hp: max_hp,
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
