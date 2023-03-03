use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
};
use vector2d::Vector2D;

use crate::consts::helper::CURRENT_ENT_ID;

use super::order::Order;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum EntParentType {
    Unit,
    OrePatch,
    Ore,
    Structure,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Owner {
    Nature,
    Player,
    Cpu,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum State {
    Alert, // No pending orders; Will latch on to closest enemy in range. Idle.
    Busy,  // Will continue to do whatever until it's done or cancelled (or death).
    Stop,  // No orders; Will remain still untill another order is directly issued by the owner.
    Hold, // No orders; Will hold position and issue LazyAttack commands (attack whatever enters its range, but won't chase)
          // -> TODO: maybe make it switch targets to closes target? will have to see
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntID(pub u64);

pub struct Ent {
    pub id: EntID,
    pub position: Vector2D<f32>,
    pub rect_size: Point,
    pub max_hp: u32,
    pub hp: f32,
    pub color: Color,
    pub owner: Owner,
    pub state: State,
    pub orders: Vec<Order>,
    selected: bool,
    parent_type: EntParentType,
}

impl Ent {
    pub fn new(
        parent_type: EntParentType,
        owner: Owner,
        max_hp: u32,
        position: Vector2D<f32>,
        rect_size: Point,
        color: Color,
    ) -> Self {
        unsafe {
            CURRENT_ENT_ID.0 += 1;
        }
        Self {
            id: unsafe { CURRENT_ENT_ID },
            position,
            rect_size,
            max_hp,
            hp: max_hp as f32,
            color,
            owner,
            state: State::Alert,
            orders: Vec::<Order>::new(),
            selected: false,
            parent_type,
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

    pub fn parent_type(&self) -> EntParentType {
        self.parent_type
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

    pub fn add_order(&mut self, new_order: Order, replace: bool) {
        if replace {
            self.clear_orders();
        }
        self.orders.push(new_order);
    }

    pub fn clear_orders(&mut self) {
        self.orders.clear();
    }

    pub fn bump_order(&mut self, new_order: Order) {
        let mut new_orders = vec![new_order];
        new_orders.append(&mut self.orders);
        self.orders = new_orders;
    }
}
