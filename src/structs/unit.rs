use std::ops::IndexMut;

use bevy::prelude::*;

use crate::TIME_STEP;

use crate::Collider;

use crate::MouseInfo;
use crate::Selection;

use crate::ent::Ent;

use crate::order::Order;
use crate::order::OrderType;

#[derive(Copy, Clone, Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Unit {
    pub ent: Ent,
    speed: f32,
    velocity: Velocity,
    pub orders: Vec<Order>,
}

impl Unit {
    pub fn new(ent: Ent, speed: f32, velocity: Velocity) -> Self {
        Self {
            ent,
            speed,
            velocity,
            orders: Vec::<Order>::new(),
        }
    }

    pub fn set_velocity(&mut self, _velocity: Velocity) {
        self.velocity = _velocity;
    }

    pub fn clear_velocity(&mut self) {
        self.velocity = Velocity(Vec2::new(0.0, 0.0));
    }

    pub fn add_order(&mut self, _order: Order, replace: bool) {
        if replace {
            self.orders.clear();
        }
        self.orders.push(_order);
    }

    // If there is an order in the vector, grab it, mark it as executed, and process it's effects
    pub fn execute_next_order(&mut self) -> (Option<&mut Order>, Option<Velocity>) {
        if self.orders.len() > 0 {
            let next_order = self.orders.index_mut(0);
            let copy_of_target = next_order.target;
            let new_velocity = Velocity(get_walk_velocity_to(
                Vec2::new(self.ent.position.x, self.ent.position.y),
                copy_of_target,
                self.speed,
            ));
            next_order.execute();
            return (Some(next_order), Some(new_velocity));
        }
        (None, None)
    }

    // This method checks the current executed order for completion
    // If its completed, marks it as so, and processes results
    // Then removes all completed orders from unit's vector
    pub fn update_orders(&mut self) {
        if self.orders.len() > 0 {
            let next_order = self.orders.index_mut(0);

            if !next_order.completed && next_order.executed {
                match next_order.order_type {
                    OrderType::Move => {
                        if next_order
                            .target
                            .abs_diff_eq(Vec2::new(self.ent.position.x, self.ent.position.y), 5.0)
                        {
                            // Mark this order as completed
                            next_order.completed = true;

                            // The unit has moved to it's target successfully
                            // If this was the last action in the queue, clear it's velocity so it can rest
                            if self.orders.len() == 1 {
                                self.clear_velocity()
                            };
                        }
                    }
                }
            }

            self.clear_unwated_orders();
        }
    }

    // This method removes completed orders from the unit's order vector
    fn clear_unwated_orders(&mut self) {
        self.orders.retain(|order| !order.completed);
    }
}

// This method returns a normalized vector with size speed that points from, to
pub fn get_walk_velocity_to(from: Vec2, to: Vec2, speed: f32) -> Vec2 {
    return (to - from).normalize() * Vec2::new(speed, speed);
}

// This method applies velocity each tick to every unit
// (also stores its transform info inside it's unit component)
pub fn apply_velocity(mut query: Query<(&mut Transform, &mut Unit)>) {
    for (mut transform, mut unit) in &mut query {
        transform.translation.x += unit.velocity.x * TIME_STEP;
        transform.translation.y += unit.velocity.y * TIME_STEP;

        // Store new transform info inside the unit
        unit.ent.position = transform.translation;
    }
}

// This method issues commands to the current selection of units
pub fn issue_orders(
    mouse_info: Res<MouseInfo>,
    keyboard_input: Res<Input<KeyCode>>,
    mut selection: ResMut<Selection>,
    mut query: Query<(Entity, &mut Unit), With<Collider>>,
) {
    if mouse_info.right_button {
        // issue move command to mouse position
        for selected_entity in &mut selection.current {
            for (entity, mut unit) in &mut query {
                if entity == *selected_entity {
                    let move_order = Order::new(OrderType::Move, mouse_info.position);
                    unit.add_order(move_order, !keyboard_input.pressed(KeyCode::LShift));
                }
            }
        }
    }
}
