use std::ops::{Index, IndexMut};

use vector2d::Vector2D;

use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::consts::helper::{draw_selection_border, draw_waypoint, get_direction_from_to};
use crate::consts::values::{
    ATTACKER_SPEED_PENALTY, BASE_UNIT_DAMAGE, BASE_UNIT_RANGE, BASE_UNIT_SPEED, BLACK_RGB, RED_RGB,
    RED_RGBA_WEAK, SELECTION_BORDER_COLOR, SELECTION_TARGET_BORDER_COLOR, TIME_STEP,
};
use crate::ent::Ent;

use crate::order::{Order, OrderType};

use super::ent::Team;
use super::order::AttackTarget;
use super::world_info::WorldInfo;

pub struct Unit {
    pub ent: Ent,
    pub speed: f32,
    pub damage: f32,
    pub range: f32,
    is_attacking: bool,
    velocity: Vector2D<f32>,
    pub orders: Vec<Order>,
}

impl Unit {
    pub fn new(ent: Ent) -> Self {
        Self {
            ent,
            speed: BASE_UNIT_SPEED,
            damage: BASE_UNIT_DAMAGE,
            range: BASE_UNIT_RANGE,
            is_attacking: false,
            velocity: Vector2D::<f32>::new(0.0, 0.0),
            orders: Vec::<Order>::new(),
        }
    }

    pub fn tick(&mut self, world_info: &mut WorldInfo) {
        // Update local HP based on world_info data
        // If not found there, then unit is dead
        self.ent.hp = world_info.get_ent_hp(&self.ent).unwrap_or(0.0);

        // If dead, return early
        if self.ent.hp <= 0.0 {
            return;
        }

        // Apply velocity (if any)
        self.apply_velocity();

        // Execute next order
        self.update_orders(world_info);
        let (next_order_option, next_order_direction_option) = self.execute_next_order();

        // If there are no orders to update, return early
        if next_order_option.is_none() {
            return;
        }

        // Update current order status
        let next_order =
            next_order_option.expect(">> Could not update next order; unit order vector is empty");
        match next_order.order_type {
            OrderType::Move => {
                self.is_attacking = false;
                self.set_velocity(next_order_direction_option.expect(
                    ">> Could not set unit velocity; current order did not produce a direction vector",
                ));
            }
            OrderType::Attack => {
                let possible_attack_target = &next_order.attack_target;
                if possible_attack_target.ent_id.is_none() {
                    return;
                }
                let attack_target_id = possible_attack_target
                    .ent_id
                    .expect(">> Could not find attack target id from current order");
                let possible_attack_target_pos =
                    world_info.get_ent_poisition_by_id(attack_target_id);
                if possible_attack_target_pos.is_none() {
                    return;
                }
                let attack_target_pos = possible_attack_target_pos
                    .expect(">> Could not find attack target position from world info");
                if (self.ent.position - attack_target_pos).length() < self.range {
                    // If target is in range, stop
                    self.clear_velocity();
                    // Mark as attacking
                    self.is_attacking = true;
                    // Try to attack
                    world_info.damage_ent(attack_target_id, self.damage * TIME_STEP);
                } else {
                    // If target is not in range, move towards it
                    self.set_velocity(next_order_direction_option.expect(">> Could not set unit velocity; current order did not produce a direction vector"));
                    // Mark as not attacking
                    self.is_attacking = false;
                }
            }
            OrderType::AttackMove => {
                // Check if any other unit is in range; if so, issue attack order to the closest one
                let mut closest_ent_in_range = AttackTarget {
                    ent_id: None,
                    ent_rect: None,
                    ent_team: None,
                };
                let mut has_target_in_range = false;
                let mut closest_ent_distance = self.range;
                for (ent_id, ent_rect_center) in &world_info.ent_rect_center {
                    if *ent_id == self.ent.id {
                        // Cannot attack self; return early
                        continue;
                    }
                    if world_info
                        .get_ent_team_by_id(*ent_id)
                        .expect(">> Could not find ent team by id")
                        == self.ent.team
                    {
                        // Cannot attack an ent on the same team; return early
                        continue;
                    }
                    let distance = (self.ent.position
                        - Vector2D::<f32>::new(ent_rect_center.x, ent_rect_center.y))
                    .length();
                    if distance > self.range {
                        // Too far away to attack; return early
                        continue;
                    }
                    // At this point, we know there is at least one target in range
                    has_target_in_range = true;
                    // Only attack the closest possible target
                    if distance < closest_ent_distance {
                        closest_ent_distance = distance;
                        closest_ent_in_range = AttackTarget {
                            ent_id: Some(*ent_id),
                            ent_rect: Some(
                                world_info
                                    .get_ent_rect_by_id(*ent_id)
                                    .expect(">> Could not find entity rect by id"),
                            ),
                            ent_team: world_info.get_ent_team_by_id(*ent_id),
                        };
                    }
                }
                if has_target_in_range {
                    let ent_rect = closest_ent_in_range
                        .ent_rect
                        .expect(">> Could not find ent rect by id");
                    let attack_order = Order::new(
                        OrderType::Attack,
                        Vector2D::<f32>::new(ent_rect.x as f32, ent_rect.y as f32),
                        closest_ent_in_range,
                    );
                    // Issue attack order to closest in-range target
                    // Bump it so that it takes precedence over this attack move order
                    self.bump_order(attack_order);
                } else {
                    // Just move towards the target position
                    self.is_attacking = false;
                    self.set_velocity(next_order_direction_option.expect(">> Could not set unit velocity; current order did not produce a direction vector"));
                }
            }
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        // If dead, return early
        if self.ent.hp <= 0.0 {
            return {};
        }
        // If selected, draw selection border
        if self.ent.selected() {
            let border_color = if self.ent.team == Team::Player {
                SELECTION_BORDER_COLOR
            } else {
                RED_RGBA_WEAK
            };
            draw_selection_border(canvas, &self.ent.get_rect(), border_color);
        }

        // Draw attack lines (if attacking)
        if self.is_attacking {
            let possible_attack_order = self.orders.get(0);
            if let Some(attack_order) = possible_attack_order {
                canvas.set_draw_color(RED_RGB);
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
        // Draw self (if alive)
        canvas.set_draw_color(self.ent.color);
        if self.ent.team == Team::Cpu {
            canvas.set_draw_color(BLACK_RGB);
        }
        let rect = self.ent.get_rect();
        canvas.fill_rect(rect).ok().unwrap_or_default();
        canvas.set_draw_color(BLACK_RGB);
        canvas.draw_rect(rect).ok();
    }

    pub fn draw_orders(&self, canvas: &mut Canvas<Window>) {
        // Draw order waypoints, if selected
        if self.ent.selected() {
            canvas.set_draw_color(Color::RGB(0, 150, 0));
            for (i, order) in self.orders.iter().enumerate() {
                // Draw lines connecting order waypoints
                // Set colors according to order type
                match order.order_type {
                    OrderType::Move => canvas.set_draw_color(Color::RGB(0, 150, 0)),
                    OrderType::Attack | OrderType::AttackMove => {
                        canvas.set_draw_color(Color::RGB(100, 0, 0))
                    }
                }
                if i == 0 {
                    // If this is the next order, draw  a line from unit to waypoint
                    canvas
                        .draw_line(
                            self.ent.get_rect().center(),
                            Point::new(order.move_target.x as i32, order.move_target.y as i32),
                        )
                        .ok()
                        .unwrap_or_default();
                }
                // Else, draw line from last waypoint to this one
                else {
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
                // Draw waypoint, if needed
                match order.order_type {
                    // In case of attack order, draw red selection border on attacked ent
                    // (if target is still alive)
                    OrderType::Attack => {
                        if let Some(attack_target_rect) = &order.attack_target.ent_rect {
                            draw_selection_border(
                                canvas,
                                attack_target_rect,
                                SELECTION_TARGET_BORDER_COLOR,
                            )
                        }
                    }
                    // In case of move or attack move order, draw waypoint
                    OrderType::Move | OrderType::AttackMove => {
                        draw_waypoint(order, canvas);
                    }
                }
            }
        }
    }

    pub fn set_velocity(&mut self, velocity: Vector2D<f32>) {
        self.velocity = velocity;
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

    pub fn add_order(&mut self, new_order: Order, replace: bool) {
        if replace {
            self.orders.clear();
        }
        self.orders.push(new_order);
    }

    pub fn bump_order(&mut self, new_order: Order) {
        let mut new_orders = vec![new_order];
        new_orders.append(&mut self.orders);
        self.orders = new_orders;
    }

    // If there is an order in the vector, grab it, mark it as executed, and process it's effects
    pub fn execute_next_order(&mut self) -> (Option<&mut Order>, Option<Vector2D<f32>>) {
        if !self.orders.is_empty() {
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
        if !self.orders.is_empty() {
            let next_order = self.orders.index_mut(0);

            if !next_order.completed && next_order.executed {
                match next_order.order_type {
                    OrderType::Attack => {
                        if !world_info.has_ent_by_id(
                            next_order
                                .attack_target
                                .ent_id
                                .expect(">> Could not find attack target id from current order"),
                        ) {
                            // Mark self as not attacking
                            self.is_attacking = false;

                            // Mark this order as completed
                            next_order.complete();

                            // The unit has moved to it's target successfully
                            // Clear it's velocity so it can rest
                            self.clear_velocity();
                        }
                    }
                    OrderType::Move | OrderType::AttackMove => {
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
