use vector2d::Vector2D;

use super::ent::EntID;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OrderType {
    Move,
    Attack,
    AttackMove,
}

pub struct Order {
    pub order_type: OrderType,
    pub executed: bool,
    pub completed: bool,
    pub move_target: Vector2D<f32>,
    pub attack_target: Option<EntID>,
}

impl Order {
    pub const fn new(
        order_type: OrderType,
        move_target: Vector2D<f32>,
        attack_target: Option<EntID>,
    ) -> Self {
        Self {
            order_type,
            executed: false,
            completed: false,
            move_target,
            attack_target,
        }
    }

    pub fn execute(&mut self) {
        self.executed = true;
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }
}
