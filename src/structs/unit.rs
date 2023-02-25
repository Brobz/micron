use std::ops::Index;
use std::ops::IndexMut;

use vector2d::Vector2D;

use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::consts::helper::get_direction_from_to;
use crate::consts::setup::ATTACKER_SPEED_PENALTY;
use crate::consts::setup::BASE_UNIT_DAMAGE;
use crate::consts::setup::BASE_UNIT_RANGE;
use crate::consts::setup::BASE_UNIT_SPEED;
use crate::ent::Ent;
use crate::TIME_STEP;

use crate::order::Order;
use crate::order::OrderType;

use super::world_info::WorldInfo;

pub struct Unit {
    pub ent: Ent,
    pub speed: f32,
    pub damage: f32,
    pub range: f32,
    selected: bool,
    is_attacking: bool,
    velocity: Vector2D<f32>,
    pub orders: Vec<Order>,
}

impl Unit {
    pub fn new(ent: Ent) -> Unit {
        Unit {
            ent,
            speed: BASE_UNIT_SPEED,
            damage: BASE_UNIT_DAMAGE,
            range: BASE_UNIT_RANGE,
            selected: false,
            is_attacking: false,
            velocity: Vector2D::<f32>::new(0.0, 0.0),
            orders: Vec::<Order>::new(),
        }
    }

    pub fn tick(&mut self, world_info: &mut WorldInfo) {
        // Update local HP based on world_info data
        self.ent.hp = world_info.get_ent_hp(&self.ent).unwrap_or(0.0);

        // Apply velocity (if any)
        self.apply_velocity();

        // Execute next order
        self.update_orders(&world_info);
        let (next_order_option, next_order_direction_option) = self.execute_next_order();
        if next_order_option.is_none() {
            return;
        }
        let next_order = next_order_option.unwrap();
        match next_order.order_type {
            OrderType::Move => {
                self.is_attacking = false;
                self.set_velocity(next_order_direction_option.unwrap());
            }
            OrderType::Attack => {
                let possible_attack_target = &next_order.attack_target;
                if possible_attack_target.is_none() {
                    return;
                }
                let attack_target = possible_attack_target.unwrap();
                let possible_attack_target_pos = world_info.get_ent_poisition_by_id(&attack_target);
                if possible_attack_target_pos.is_none() {
                    return;
                }
                let attack_target_pos = possible_attack_target_pos.unwrap();
                if (self.ent.position - attack_target_pos).length() < self.range as f32 {
                    // If target is in range, stop
                    self.clear_velocity();
                    // Mark as attacking
                    self.is_attacking = true;
                    // Try to attack
                    world_info.damage_ent(&attack_target, self.damage * TIME_STEP);
                } else {
                    // If target is not in range, move towards it
                    self.set_velocity(next_order_direction_option.unwrap());
                    // Mark as not attacking
                    self.is_attacking = false;
                }
            }
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        // Draw order waypoints, if selected
        if self.selected {
            canvas.set_draw_color(Color::RGB(0, 150, 0));
            for (i, order) in self.orders.iter().enumerate() {
                // Draw lines connecting order waypoints
                // Set colors according to order type
                match order.order_type {
                    OrderType::Move => canvas.set_draw_color(Color::RGB(0, 150, 0)),
                    OrderType::Attack => canvas.set_draw_color(Color::RGB(150, 0, 0)),
                }
                match i {
                    // If this is the next order, draw  a line from unit to waypoint
                    0 => {
                        canvas
                            .draw_line(
                                self.ent.get_rect().center(),
                                Point::new(order.move_target.x as i32, order.move_target.y as i32),
                            )
                            .ok()
                            .unwrap_or_default();
                    }
                    // Else, draw line from last waypoint to this one
                    _ => {
                        let previous_order_target = self.orders.index(i - 1).move_target;
                        canvas
                            .draw_line(
                                Point::new(
                                    previous_order_target.x as i32,
                                    previous_order_target.y as i32,
                                ),
                                Point::new(order.move_target.x as i32, order.move_target.y as i32),
                            )
                            .ok()
                            .unwrap_or_default();
                    }
                }

                match order.order_type {
                    // In case of move order, draw waypoint
                    OrderType::Move => {
                        let waypoint_rect: Rect = Rect::from_center(
                            Point::new(order.move_target.x as i32, order.move_target.y as i32),
                            5,
                            5,
                        );
                        canvas.fill_rect(waypoint_rect).ok().unwrap_or_default();
                    }
                    // In case of attack order, nothing for now
                    OrderType::Attack => (),
                }
            }
        } else if self.is_attacking {
            // Draw attack lines (if attacking)
            let possible_attack_order = self.orders.get(0);
            if possible_attack_order.is_some() {
                let attack_order = possible_attack_order.unwrap();
                canvas.set_draw_color(Color::RED);
                canvas
                    .draw_line(
                        self.ent.get_rect().center(),
                        Point::new(
                            attack_order.move_target.x as i32,
                            attack_order.move_target.y as i32,
                        ),
                    )
                    .ok()
                    .unwrap_or_default();
            }
        }

        // If selected, draw selection border
        if self.selected {
            canvas.set_draw_color(Color::RGB(80, 200, 80));
            let selection_border_rect: Rect = Rect::new(
                (self.ent.position.x - 3.0) as i32,
                (self.ent.position.y - 3.0) as i32,
                (self.ent.rect_size.x + 6) as u32,
                (self.ent.rect_size.y + 6) as u32,
            );
            canvas
                .fill_rect(selection_border_rect)
                .ok()
                .unwrap_or_default();
        }

        // Draw self (if alive)
        canvas.set_draw_color(Color::RGB(50, 10, 50));
        let rect: Rect = Rect::new(
            self.ent.position.x as i32,
            self.ent.position.y as i32,
            self.ent.rect_size.x as u32,
            self.ent.rect_size.y as u32,
        );
        if self.ent.hp > 0.0 {
            canvas.fill_rect(rect).ok().unwrap_or_default();
        }
    }

    pub fn set_velocity(&mut self, _velocity: Vector2D<f32>) {
        self.velocity = _velocity;
    }

    pub fn clear_velocity(&mut self) {
        self.velocity = Vector2D::<f32>::new(0.0, 0.0);
    }

    // This method applies velocity each tick to the unit
    pub fn apply_velocity(&mut self) {
        let attack_penalty: f32 = if self.is_attacking {
            ATTACKER_SPEED_PENALTY
        } else {
            1.0
        };
        self.ent.position.x += self.velocity.x * TIME_STEP * attack_penalty;
        self.ent.position.y += self.velocity.y * TIME_STEP * attack_penalty;
    }

    pub fn add_order(&mut self, _order: Order, replace: bool) {
        if replace {
            self.orders.clear();
        }
        self.orders.push(_order);
    }

    // If there is an order in the vector, grab it, mark it as executed, and process it's effects
    pub fn execute_next_order(&mut self) -> (Option<&mut Order>, Option<Vector2D<f32>>) {
        if self.orders.len() > 0 {
            let next_order = self.orders.index_mut(0);
            let copy_of_target = next_order.move_target;
            let rect_center = self.ent.get_rect().center();
            let new_velocity = get_direction_from_to(
                Vector2D::<f32>::new(rect_center.x as f32, rect_center.y as f32),
                copy_of_target,
                self.speed,
            );
            next_order.execute();
            return (Some(next_order), Some(new_velocity));
        }
        (None, None)
    }

    // This method checks the current executed order for completion
    // If its completed, marks it as so, and processes results
    // Then removes all completed orders from unit's vector
    pub fn update_orders(&mut self, world_info: &WorldInfo) {
        if self.orders.len() > 0 {
            let next_order = self.orders.index_mut(0);

            if !next_order.completed && next_order.executed {
                match next_order.order_type {
                    OrderType::Move => {
                        let rect_center = self.ent.get_rect().center();
                        if (next_order.move_target
                            - Vector2D::<f32>::new(rect_center.x as f32, rect_center.y as f32))
                        .length()
                            <= 3.0
                        {
                            // Mark this order as completed
                            next_order.complete();

                            // The unit has moved to it's target successfully
                            // Clear it's velocity so it can rest
                            self.clear_velocity();
                        }
                    }
                    OrderType::Attack => {
                        if !world_info.has_ent_by_id(&next_order.attack_target.unwrap()) {
                            // Mark self as not attacking
                            self.is_attacking = false;

                            // Mark this order as completed
                            next_order.complete();

                            // The unit has moved to it's target successfully
                            // Clear it's velocity so it can rest
                            self.clear_velocity();
                        }
                    }
                }
            }

            self.clear_unwated_orders();
        }
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn select(&mut self) {
        self.selected = true
    }

    pub fn deselect(&mut self) {
        self.selected = false
    }

    // This method removes completed orders from the unit's order vector
    fn clear_unwated_orders(&mut self) {
        self.orders.retain(|order| !order.completed);
    }
}
