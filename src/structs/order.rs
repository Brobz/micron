use sdl2::rect::Rect;
use vector2d::Vector2D;

use super::ent::{EntID, Owner};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OrderType {
    Move,
    Follow,
    Attack,
    AttackMove,
}

pub struct Order {
    pub order_type: OrderType,
    pub executed: bool,
    pub completed: bool,
    pub current_move_target: Vector2D<f32>,
    pub ent_target: EntTarget,
}

pub struct EntTarget {
    pub ent_id: Option<EntID>,
    pub ent_rect: Option<Rect>,
    pub ent_owner: Option<Owner>,
}

impl Order {
    pub const fn new(
        order_type: OrderType,
        current_move_target: Vector2D<f32>,
        ent_target: EntTarget,
    ) -> Self {
        Self {
            order_type,
            executed: false,
            completed: false,
            current_move_target,
            ent_target,
        }
    }

    pub fn execute(&mut self) {
        self.executed = true;
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }
}
