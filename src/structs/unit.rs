use std::ops::{Index, IndexMut};

use rand::Rng;
use vector2d::Vector2D;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::consts::helper::{draw_selection_border, draw_waypoint, get_direction_from_to};
use crate::consts::values::{
    ATTACKER_SPEED_PENALTY, BASE_UNIT_DAMAGE, BASE_UNIT_RANGE, BASE_UNIT_SPEED, BLACK_RGB,
    FOLLOW_ORDER_HOVER_DISTANCE, GREY_RGB, ORANGE_RGB, RED_RGBA_WEAK,
    SELECTION_ATTACK_TARGET_BORDER_COLOR, SELECTION_BORDER_COLOR,
    SELECTION_FOLLOW_TARGET_BORDER_COLOR, TIME_STEP,
};
use crate::ent::Ent;

use crate::order::{Order, OrderType};

use super::ent::{Owner, State};
use super::order::EntTarget;
use super::world_info::WorldInfo;

pub struct Unit {
    pub ent: Ent,
    pub speed: f32,
    pub damage: f32,
    pub range: f32,
    is_attacking: bool,
    attack_line_render_latch_point_delta: Option<Point>,
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
            attack_line_render_latch_point_delta: None,
            velocity: Vector2D::<f32>::new(0.0, 0.0),
            orders: Vec::<Order>::new(),
        }
    }

    // TODO: CLEAN UP THIS METHOD
    //          -> Separate into smaller methods
    //          -> Abstract out repeated logic sections
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

        // Check for Stop state
        // If there is no next order to execute
        if self.orders.is_empty() {
            // If stopped, return early
            if self.ent.state == State::Stop {
                return;
            }
            // Else, if not holding, set state to Alert
            else if self.ent.state != State::Hold {
                self.ent.state = State::Alert;
            }
        }

        // Checks for Alert & Hold state
        // An alert unit has no pending orders in its queue
        // It should actively seek to attack enemy units that appear in its range
        // Hold units behave similarly, but issue lazy attacks instead (won't chase)
        if vec![State::Alert, State::Hold].contains(&self.ent.state) {
            // Check if any other unit is in range; if so, issue attack order to the closest one
            let mut closest_ent_in_range = EntTarget {
                ent_id: None,
                ent_rect: None,
                ent_owner: None,
            };
            let mut has_target_in_range = false;
            let mut closest_ent_distance = self.range;
            for (ent_id, ent_rect_center) in &world_info.ent_rect_center {
                if *ent_id == self.ent.id {
                    // Cannot attack self; return early
                    continue;
                }
                if world_info
                    .get_ent_owner_by_id(*ent_id)
                    .expect(">> Could not find ent team by id")
                    == self.ent.owner
                {
                    // Cannot attack an ent on the same team; return early
                    continue;
                }
                let self_rect_center = self.ent.get_rect().center();
                let distance =
                    (Vector2D::<f32>::new(self_rect_center.x as f32, self_rect_center.y as f32)
                        - Vector2D::<f32>::new(ent_rect_center.x, ent_rect_center.y))
                    .length();
                if distance > self.range {
                    // Too far away to attack; return early
                    continue;
                }
                // Only attack the closest possible target
                if distance < closest_ent_distance {
                    // At this point, we know there is at least one target in range
                    has_target_in_range = true;
                    closest_ent_distance = distance;
                    closest_ent_in_range = EntTarget {
                        ent_id: Some(*ent_id),
                        ent_rect: Some(
                            world_info
                                .get_ent_rect_by_id(*ent_id)
                                .expect(">> Could not find entity rect by id"),
                        ),
                        ent_owner: world_info.get_ent_owner_by_id(*ent_id),
                    };
                }
            }

            if has_target_in_range {
                let ent_rect = closest_ent_in_range
                    .ent_rect
                    .expect(">> Could not find ent rect by id");
                let attack_order = Order::new(
                    if self.ent.state == State::Alert {
                        OrderType::Attack
                    } else {
                        OrderType::LazyAttack
                    },
                    Vector2D::<f32>::new(ent_rect.x as f32, ent_rect.y as f32),
                    closest_ent_in_range,
                );
                // Issue attack order to closest in-range target
                // Bump it so that it takes precedence over this attack move order
                self.bump_order(attack_order);
            }
        }

        // Check existing orders for completion
        self.check_orders();

        // Clear any completed orders
        self.clear_completed_orders();

        // If no orders, return early
        if self.orders.is_empty() {
            return;
        }

        // Try to grab next order
        let (next_order_option, next_order_direction_option) = self.grab_next_order();

        // Execute current order
        let next_order =
            next_order_option.expect(">> Could not grab next order from unit order vector");
        // Keep a flag in case we can safely clear next order after execution
        let mut did_complete_order = false;

        match next_order.order_type {
            OrderType::Move => {
                self.ent.state = State::Busy;
                self.stop_attacking();
                self.set_velocity(next_order_direction_option.expect(
                    ">> Could not set unit velocity; current order did not produce a direction vector",
                ));
            }
            OrderType::Attack => {
                let possible_attack_target = &next_order.ent_target;
                if possible_attack_target.ent_id.is_none() {
                    // No more target, attack is done!
                    did_complete_order = true;
                    // Mark self as not attacking
                    self.stop_attacking();
                    // Clear velocity; Attack order could have a unit moving
                    self.clear_velocity();
                } else {
                    let attack_target_id = possible_attack_target
                        .ent_id
                        .expect(">> Could not find attack target id from current order");
                    if !world_info.has_ent_by_id(attack_target_id) {
                        // No more target, attack is done!
                        did_complete_order = true;
                        // Mark self as not attacking
                        self.stop_attacking();
                        // Clear velocity; Attack order could have a unit moving
                        self.clear_velocity();
                    } else {
                        let attack_target_pos = world_info
                            .get_ent_rect_center_poisition_by_id(attack_target_id)
                            .expect(">> Could not find attack target position from world info");
                        let self_rect_center = self.ent.get_rect().center();
                        if (Vector2D::<f32>::new(
                            self_rect_center.x as f32,
                            self_rect_center.y as f32,
                        ) - attack_target_pos)
                            .length()
                            < self.range
                        {
                            // If target is in range, check if already attacking
                            if self.is_attacking {
                                world_info.damage_ent(attack_target_id, self.damage * TIME_STEP);
                            } else {
                                // Else, start attacking
                                self.start_attacking(possible_attack_target.ent_rect.expect(">> Could not get ent rect from attack target, but could find rect center position in world info?"));
                            }
                        } else {
                            // If target is not in range, move towards it
                            self.set_velocity(next_order_direction_option.expect(">> Could not set unit velocity; current order did not produce a direction vector"));
                            // Mark as not attacking
                            self.stop_attacking()
                        }
                        self.ent.state = State::Busy;
                    }
                }
            }
            OrderType::AttackMove => {
                self.ent.state = State::Alert;
                self.stop_attacking();
                self.set_velocity(next_order_direction_option.expect(">> Could not set unit velocity; current order did not produce a direction vector"));
            }
            OrderType::Follow => {
                // Unit should stop moving if it gets within a certain distance of it's follow target
                // If target is dead, complete order
                if next_order.ent_target.ent_id.is_none() {
                    did_complete_order = true;
                } else {
                    let follow_target_id = next_order
                        .ent_target
                        .ent_id
                        .expect(">> Could not get entity id form follow target");
                    if !world_info.has_ent_by_id(follow_target_id) {
                        did_complete_order = true;
                    }
                }
                let rect_center = self.ent.get_rect().center();
                if (next_order.current_move_target
                    - Vector2D::<f32>::new(rect_center.x as f32, rect_center.y as f32))
                .length()
                    <= FOLLOW_ORDER_HOVER_DISTANCE
                {
                    self.clear_velocity();
                } else {
                    self.set_velocity(next_order_direction_option.expect(
                        ">> Could not set unit velocity; current order did not produce a direction vector",
                    ));
                }
                self.stop_attacking();
                self.ent.state = State::Busy;
            }
            OrderType::LazyAttack => {
                let possible_attack_target = &next_order.ent_target;
                if possible_attack_target.ent_id.is_none() {
                    // No more target, Lazy Attack is done!
                    // Mark self as not attacking as well
                    self.stop_attacking();
                    did_complete_order = true;
                    // Note: Since lazy attacks only occur while holding poisitiong (FOR NOW),
                    // We must set this unit's state from Busy back to Hold after its done with the lazy attack
                    self.ent.state = State::Hold;
                } else {
                    let attack_target_id = possible_attack_target
                        .ent_id
                        .expect(">> Could not find attack target id from current order");
                    if !world_info.has_ent_by_id(attack_target_id) {
                        // No more target, Lazy Attack is done!
                        // Mark self as not attacking as well
                        self.stop_attacking();
                        did_complete_order = true;
                        // Note: Since lazy attacks only occur while holding poisitiong (FOR NOW),
                        // We must set this unit's state from Busy back to Hold after its done with the lazy attack
                        self.ent.state = State::Hold;
                    } else {
                        let attack_target_pos = world_info
                            .get_ent_rect_center_poisition_by_id(attack_target_id)
                            .expect(">> Could not find attack target position from world info");
                        let self_rect_center = self.ent.get_rect().center();
                        if (Vector2D::<f32>::new(
                            self_rect_center.x as f32,
                            self_rect_center.y as f32,
                        ) - attack_target_pos)
                            .length()
                            < self.range
                        {
                            // If target is in range, check if already attacking
                            if self.is_attacking {
                                // Already attacking, try to deal damage
                                world_info.damage_ent(attack_target_id, self.damage * TIME_STEP);
                            } else {
                                // Else, start attacking
                                self.start_attacking(
                                    world_info.get_ent_rect_by_id(attack_target_id).expect(
                                        ">> Could not get entity rect by id from attack target",
                                    ),
                                );
                            }
                        } else {
                            // If not in range, Lazy Attack is done!
                            did_complete_order = true;
                            self.stop_attacking();
                            // Note: Since lazy attacks only occur while holding poisitiong (FOR NOW),
                            // We must set this unit's state from Busy back to Hold after its done with the lazy attack
                            self.ent.state = State::Hold;
                        }
                    }
                }
            }
            OrderType::HoldPosition => {
                self.hold_position();
                did_complete_order = true;
            }
        }

        // Check if we can complete the order
        if did_complete_order {
            self.orders.index_mut(0).complete();
        } else {
            // Else, mark order as executed
            self.orders.index_mut(0).execute();
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        // If dead, return early
        if self.ent.hp <= 0.0 {
            return {};
        }
        // If selected, draw selection border
        if self.ent.selected() {
            let border_color = if self.ent.owner == Owner::Player {
                SELECTION_BORDER_COLOR
            } else {
                RED_RGBA_WEAK
            };
            draw_selection_border(canvas, &self.ent.get_rect(), border_color);
        }

        // Draw self (if alive)
        canvas.set_draw_color(self.ent.color);
        if self.ent.owner == Owner::Cpu {
            canvas.set_draw_color(BLACK_RGB);
        }
        let rect = self.ent.get_rect();
        canvas.fill_rect(rect).ok().unwrap_or_default();
        canvas.set_draw_color(BLACK_RGB);
        canvas.draw_rect(rect).ok();
        if self.ent.state == State::Stop {
            canvas.set_draw_color(GREY_RGB);
            canvas.draw_point(self.ent.get_rect().center()).ok();
        }
        if self.ent.state == State::Hold {
            canvas.set_draw_color(ORANGE_RGB);
            canvas.draw_point(self.ent.get_rect().center()).ok();
        }
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
                    OrderType::Attack | OrderType::AttackMove | OrderType::LazyAttack => {
                        canvas.set_draw_color(SELECTION_ATTACK_TARGET_BORDER_COLOR)
                    }
                    OrderType::Follow => {
                        canvas.set_draw_color(SELECTION_FOLLOW_TARGET_BORDER_COLOR)
                    }
                    OrderType::HoldPosition => canvas.set_draw_color(ORANGE_RGB),
                }
                if i == 0 {
                    // If this is the next order, draw  a line from unit to waypoint
                    canvas
                        .draw_line(
                            self.ent.get_rect().center(),
                            Point::new(
                                order.current_move_target.x as i32,
                                order.current_move_target.y as i32,
                            ),
                        )
                        .ok()
                        .unwrap_or_default();
                }
                // Else, draw line from last waypoint to this one
                else {
                    let previous_order_target = self.orders.index(i - 1).current_move_target;
                    canvas
                        .draw_line(
                            Point::new(
                                previous_order_target.x as i32,
                                previous_order_target.y as i32,
                            ),
                            Point::new(
                                order.current_move_target.x as i32,
                                order.current_move_target.y as i32,
                            ),
                        )
                        .ok()
                        .unwrap_or_default();
                }
                // Draw waypoint, if needed
                match order.order_type {
                    // In case of attack order, draw red selection border on attacked ent
                    // (if target is still alive)
                    OrderType::Attack | OrderType::LazyAttack => {
                        if let Some(attack_target_rect) = &order.ent_target.ent_rect {
                            draw_selection_border(
                                canvas,
                                attack_target_rect,
                                SELECTION_ATTACK_TARGET_BORDER_COLOR,
                            )
                        }
                    }
                    // In case of move or attack move order, draw waypoint
                    OrderType::Move | OrderType::AttackMove => {
                        draw_waypoint(order, canvas);
                    }
                    // In case of follow order, draw yellow selection border on followed ent
                    // (if target is still alive)
                    OrderType::Follow => {
                        if let Some(follow_target_rect) = &order.ent_target.ent_rect {
                            draw_selection_border(
                                canvas,
                                follow_target_rect,
                                SELECTION_FOLLOW_TARGET_BORDER_COLOR,
                            )
                        }
                    }
                    // In case of hold position, do nothing for now
                    OrderType::HoldPosition => draw_waypoint(order, canvas),
                }
            }
        }
    }

    pub fn draw_attack_lines(&self, canvas: &mut Canvas<Window>) {
        // Draw attack lines (if attacking)
        if self.is_attacking {
            let possible_attack_order = self.orders.get(0);
            if let Some(attack_order) = possible_attack_order {
                if let Some(attack_target_rect) = attack_order.ent_target.ent_rect {
                    canvas.set_draw_color(self.ent.color);
                    canvas
                        .draw_line(
                            self.ent.get_rect().center(),
                            attack_target_rect.center() + self.attack_line_render_latch_point_delta.expect(">> Could not get attack line render latch point, but is unit attacking?"),
                        )
                        .ok()
                        .unwrap_or_default();
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

    // If there is an order in the vector, grab it
    pub fn grab_next_order(&self) -> (Option<&Order>, Option<Vector2D<f32>>) {
        if !self.orders.is_empty() {
            let next_order = self.orders.index(0);
            let copy_of_target = next_order.current_move_target;
            let rect_center = self.ent.get_rect().center();
            let new_velocity = get_direction_from_to(
                Vector2D::<f32>::new(rect_center.x as f32, rect_center.y as f32),
                copy_of_target,
                self.speed,
            );
            return (Some(next_order), Some(new_velocity));
        }
        (None, None)
    }

    // This method checks the current executed order for completion
    // If its completed, marks it as so, and processes results
    pub fn check_orders(&mut self) {
        if !self.orders.is_empty() {
            let next_order = self.orders.index_mut(0);

            if !next_order.completed && next_order.executed {
                match next_order.order_type {
                    OrderType::Move | OrderType::AttackMove => {
                        // To complete either a move or attack move order, unit must reach it's destination
                        let rect_center = self.ent.get_rect().center();
                        if (next_order.current_move_target
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
                    // A follow order can never be completed!
                    // It can only get cleard or canceled (if the followed unit dies)
                    // To complete an attack or lazy attack order, the target must be DEAD!
                    // That gets checked right before trying to attack it during execution,
                    // So it can get completed and cleaned up there as well.
                    // Same thing with HoldPosition, gets cleared right after execution.
                    OrderType::LazyAttack
                    | OrderType::Attack
                    | OrderType::Follow
                    | OrderType::HoldPosition => (),
                }
            }
        }
    }

    // This method removes completed orders from the unit's order vector
    fn clear_completed_orders(&mut self) {
        self.orders.retain(|order| !order.completed);
    }

    pub fn clear_all_but_current_order(&mut self) {
        if self.orders.len() > 1 {
            self.orders.drain(1..self.orders.len());
        }
    }

    // This method executes a stop order to the unit
    // Stop order clears velocity, falsifies attack flag, and sets state to Stop
    // It also clears orders, meaning it removes itself. pretty handy.
    pub fn stop(&mut self) {
        // Cancel all orders
        self.clear_orders();
        // Set state to stopped
        self.stop_attacking();
        self.ent.state = State::Stop;
        self.clear_velocity();
    }

    // This method executes a hold position order to the unit
    // Hold position order clears velocity, and sets state to Hold
    // It also clears orders, meaning it removes itself. pretty handy.
    fn hold_position(&mut self) {
        // Cancel all orders, but the hold position order so it gets cleared normally
        self.clear_all_but_current_order();
        // Stop moving
        self.clear_velocity();
        // Set state to hold
        self.ent.state = State::Hold;
    }

    pub fn stop_attacking(&mut self) {
        self.is_attacking = false;
        self.attack_line_render_latch_point_delta = None;
    }

    pub fn start_attacking(&mut self, attack_target_rect: Rect) {
        self.clear_velocity();
        self.ent.state = State::Busy;
        let mut rng = rand::thread_rng();
        self.is_attacking = true;
        self.attack_line_render_latch_point_delta = Some(Point::new(
            rng.gen_range(
                (-(attack_target_rect.width() as f32 + 2.0) / 2.0) as i32
                    ..((attack_target_rect.width() as f32 - 2.0) / 2.0) as i32,
            ),
            rng.gen_range(
                (-(attack_target_rect.height() as f32 + 2.0) / 2.0) as i32
                    ..((attack_target_rect.height() as f32 - 2.0) / 2.0) as i32,
            ),
        ));
    }
}
