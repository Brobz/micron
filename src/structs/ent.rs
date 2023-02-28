use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
};
use vector2d::Vector2D;

use crate::consts::helper::CURRENT_ENT_ID;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Owner {
    Player,
    Cpu,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum State {
    Alert, // No pending orders; Will latch on to closest enemy in range. Idle.
    Busy,  // Will continue to do whatever until it's done or cancelled (or death).
    Stop,  // No orders; Will remain still untill another order is directly issued by the owner.
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntID(pub u64);

pub struct Ent {
    pub id: EntID,
    pub position: Vector2D<f32>,
    pub rect_size: Point,
    pub radius: i32,
    pub max_hp: u32,
    pub hp: f32,
    pub color: Color,
    pub owner: Owner,
    pub state: State,
    selected: bool,
}

impl Ent {
    pub fn new(owner: Owner, max_hp: u32, position: Vector2D<f32>, rect_size: Point) -> Self {
        unsafe {
            CURRENT_ENT_ID.0 += 1;
        }
        Self {
            id: unsafe { CURRENT_ENT_ID },
            position,
            rect_size,
            radius: -1,
            max_hp,
            hp: max_hp as f32,
            color: Color::RGB(0, 100, 100),
            owner,
            state: State::Alert,
            selected: false,
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

    pub const fn selected(&self) -> bool {
        self.selected
    }

    pub fn select(&mut self) {
        self.selected = true;
    }

    pub fn deselect(&mut self) {
        self.selected = false;
    }
}
