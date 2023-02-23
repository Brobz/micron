use bevy::prelude::{Entity, Query, Vec2};

use crate::unit::Unit;

pub enum OrderType {
    Move,
}

pub struct Order {
    pub order_type: OrderType,
    pub executed: bool,
    pub completed: bool,
    pub target: Vec2,
}

impl Order {
    pub fn new(order_type: OrderType, target: Vec2) -> Self {
        Self {
            order_type,
            executed: false,
            completed: false,
            target,
        }
    }

    pub fn execute(&mut self) {
        self.executed = true;
    }
}

// Executes orders for all units
pub fn execute_orders(mut query: Query<(Entity, &mut Unit)>) {
    for (_, mut unit) in &mut query {
        let (next_order_option, next_order_direction_option) = unit.execute_next_order();
        if next_order_option.is_some() {
            let next_order = next_order_option.unwrap();

            match next_order.order_type {
                OrderType::Move => unit.set_velocity(next_order_direction_option.unwrap()),
            }
        }
        unit.update_orders();
    }
}
