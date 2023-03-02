use std::ops::{Index, IndexMut};

use rand::Rng;
use vector2d::Vector2D;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::consts::helper::{
    draw_selection_border, draw_waypoint, empty_ent_target, get_direction_from_to,
};
use crate::consts::values::{
    ATTACKER_SPEED_PENALTY, BASE_UNIT_DAMAGE, BASE_UNIT_MASS, BASE_UNIT_RANGE, BASE_UNIT_SPEED,
    BLACK_RGB, FOLLOW_ORDER_HOVER_DISTANCE, GREY_RGB, MAX_MOVE_ORDER_ERROR, ORANGE_RGB,
    RED_RGBA_WEAK, SELECTION_ATTACK_TARGET_BORDER_COLOR, SELECTION_BORDER_COLOR,
    SELECTION_FOLLOW_TARGET_BORDER_COLOR, SELECTION_MINE_TARGET_BORDER_COLOR, TIME_STEP,
};
use crate::ent::Ent;

use crate::order::{Order, OrderType};

use super::ent::{EntID, EntParentType, Owner, State};
use super::order::EntTarget;
use super::world_info::WorldInfo;

pub struct Unit {
    pub speed: f32,
    pub damage: f32,
    pub range: f32,
    is_attacking: bool,
    attack_line_render_latch_point_delta: Option<Point>,
    velocity: Vector2D<f32>,
    desired_velocity: Vector2D<f32>,
    mass: f32,
}

impl Unit {
    pub fn new() -> Self {
        Self {
            speed: BASE_UNIT_SPEED,
            damage: BASE_UNIT_DAMAGE,
            range: BASE_UNIT_RANGE,
            is_attacking: false,
            attack_line_render_latch_point_delta: None,
            velocity: Vector2D::<f32>::new(0.0, 0.0),
            desired_velocity: Vector2D::<f32>::new(0.0, 0.0),
            mass: BASE_UNIT_MASS,
        }
    }

    pub fn tick(&mut self, ent: &mut Ent, world_info: &mut WorldInfo) {
        // Update local HP based on world_info data
        // If not found there, then unit is dead
        ent.hp = world_info.get_ent_hp(ent).unwrap_or(0.0);

        // If dead, return early
        if ent.hp <= 0.0 {
            return;
        }

        // Apply steering
        // Steering allows a unit to go from its current velocity to target velocity, if needed
        self.apply_steering();

        // Apply velocity (if any)
        // Also handles collision detection
        self.apply_velocity(ent, world_info);

        // Check for Stop state
        // If there is no next order to execute
        if ent.orders.is_empty() {
            // If stopped, return early
            if ent.state == State::Stop {
                return;
            }
            // Else, if not holding, set state to Alert
            else if ent.state != State::Hold {
                ent.state = State::Alert;
            }
        }

        // If able, observe surroundings and take appropriate actions
        self.check_surroundings(ent, world_info);

        // Check existing orders for completion
        self.check_orders(ent);

        // Clear any completed orders
        self.clear_completed_orders(ent);

        // If no orders, return early
        if ent.orders.is_empty() {
            return;
        }

        // Try to grab next order
        let (next_order_option, next_order_direction_option) = self.grab_next_order(ent);

        // Execute current order
        let next_order =
            next_order_option.expect(">> Could not grab next order from unit order vector");
        // Keep a flag to mark completion
        let did_complete_order =
            self.execute_next_order(ent, next_order, next_order_direction_option, world_info);

        // Check if we can complete the order
        if did_complete_order {
            // Positive, mark as completed
            ent.orders.index_mut(0).complete();
        } else {
            // Else, just mark order as executed
            ent.orders.index_mut(0).execute();
        }
    }

    pub fn draw(&self, ent: &mut Ent, canvas: &mut Canvas<Window>) {
        // If dead, return early
        if ent.hp <= 0.0 {
            return {};
        }
        // If selected, draw selection border
        if ent.selected() {
            let border_color = if ent.owner == Owner::Player {
                SELECTION_BORDER_COLOR
            } else {
                RED_RGBA_WEAK
            };
            draw_selection_border(canvas, &ent.get_rect(), border_color);
        }

        // Draw self (if alive)
        canvas.set_draw_color(ent.color);
        if ent.owner == Owner::Cpu {
            canvas.set_draw_color(BLACK_RGB);
        }
        let rect = ent.get_rect();
        canvas.fill_rect(rect).ok().unwrap_or_default();
        canvas.set_draw_color(BLACK_RGB);
        canvas.draw_rect(rect).ok();
        if ent.state == State::Stop {
            canvas.set_draw_color(GREY_RGB);
            canvas.draw_point(ent.get_rect().center()).ok();
        }
        if ent.state == State::Hold {
            canvas.set_draw_color(ORANGE_RGB);
            canvas.draw_point(ent.get_rect().center()).ok();
        }
    }

    pub fn draw_orders(&self, ent: &mut Ent, canvas: &mut Canvas<Window>) {
        // Draw order waypoints, if selected
        if !ent.selected() {
            // Not selected, return early
            return;
        }

        canvas.set_draw_color(Color::RGB(0, 150, 0));
        for (i, order) in ent.orders.iter().enumerate() {
            // Draw lines connecting order waypoints
            // Set colors according to order type
            match order.order_type {
                OrderType::Move => canvas.set_draw_color(Color::RGB(0, 150, 0)),
                OrderType::Attack | OrderType::AttackMove | OrderType::LazyAttack => {
                    canvas.set_draw_color(SELECTION_ATTACK_TARGET_BORDER_COLOR)
                }
                OrderType::Follow => canvas.set_draw_color(SELECTION_FOLLOW_TARGET_BORDER_COLOR),
                OrderType::HoldPosition => canvas.set_draw_color(ORANGE_RGB),
                OrderType::Mine => canvas.set_draw_color(SELECTION_MINE_TARGET_BORDER_COLOR),
            }
            if i == 0 {
                // If this is the next order, draw  a line from unit to waypoint
                canvas
                    .draw_line(
                        ent.get_rect().center(),
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
                let previous_order_target = ent.orders.index(i - 1).current_move_target;
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
                    draw_waypoint(*order, canvas);
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
                // In case of hold position, draw waypoint
                OrderType::HoldPosition => draw_waypoint(*order, canvas),
                // In case of mining, draw white selection border on mine target
                OrderType::Mine => {
                    if let Some(mine_target_rect) = &order.ent_target.ent_rect {
                        draw_selection_border(
                            canvas,
                            mine_target_rect,
                            SELECTION_MINE_TARGET_BORDER_COLOR,
                        )
                    }
                }
            }
        }
    }

    pub fn draw_attack_lines(&self, ent: &mut Ent, canvas: &mut Canvas<Window>) {
        // Draw attack lines (if attacking)
        if self.is_attacking {
            let possible_attack_order = ent.orders.get(0);
            if let Some(attack_order) = possible_attack_order {
                if let Some(attack_target_rect) = attack_order.ent_target.ent_rect {
                    canvas.set_draw_color(ent.color);
                    canvas
                        .draw_line(
                            ent.get_rect().center(),
                            attack_target_rect.center() + self.attack_line_render_latch_point_delta.expect(">> Could not get attack line render latch point, but is unit attacking?"),
                        )
                        .ok()
                        .unwrap_or_default();
                }
            }
        }
    }

    pub fn clear_velocity(&mut self) {
        self.desired_velocity = Vector2D::<f32>::new(0.0, 0.0);
    }

    // This method applies velocity each tick to the unit
    fn apply_velocity(&mut self, ent: &mut Ent, world_info: &mut WorldInfo) {
        // Calculate speed penalty
        let attack_penalty: f32 = if self.is_attacking {
            ATTACKER_SPEED_PENALTY
        } else {
            1.0
        };
        // Apply velocity components individually in order to smoothly resolve collisions
        self.apply_x_velocity(
            ent,
            world_info,
            self.velocity.x * TIME_STEP * attack_penalty,
        );
        self.apply_y_velocity(
            ent,
            world_info,
            self.velocity.y * TIME_STEP * attack_penalty,
        );
    }

    fn apply_x_velocity(&mut self, ent: &mut Ent, world_info: &mut WorldInfo, x_velocity: f32) {
        // Aply velocity component
        ent.position.x += x_velocity;
        // Resolve collisions to the sides
        for (ent_id, ent_rect) in world_info.ent_rect.iter_mut() {
            if *ent_id == ent.id {
                continue;
            }
            if !ent.get_rect().has_intersection(*ent_rect) {
                continue;
            }
            // NO PUSH

            if ent.get_rect().has_intersection(*ent_rect) {
                if self.velocity.x > 0.0 {
                    ent.position.x = (ent_rect.left() - ent.rect_size.x) as f32;
                } else {
                    ent.position.x = ent_rect.right() as f32;
                }
            }
        }
    }

    fn apply_y_velocity(&mut self, ent: &mut Ent, world_info: &mut WorldInfo, y_velocity: f32) {
        // Aply velocity component
        ent.position.y += y_velocity;
        // Resolve collisions to top/bottom
        for (ent_id, ent_rect) in world_info.ent_rect.iter_mut() {
            if *ent_id == ent.id {
                continue;
            }
            if !ent.get_rect().has_intersection(*ent_rect) {
                continue;
            }
            if ent.get_rect().has_intersection(*ent_rect) {
                if self.velocity.y > 0.0 {
                    ent.position.y = (ent_rect.top() - ent.rect_size.y) as f32;
                } else {
                    ent.position.y = ent_rect.bottom() as f32;
                }
            }
        }
    }

    // If there is an order in the vector, grab it
    pub fn grab_next_order(&self, ent: &mut Ent) -> (Option<Order>, Option<Vector2D<f32>>) {
        if !ent.orders.is_empty() {
            let next_order = ent.orders.index(0);
            let copy_of_target = next_order.current_move_target;
            let rect_center = ent.get_rect().center();
            let new_velocity = get_direction_from_to(
                Vector2D::<f32>::new(rect_center.x as f32, rect_center.y as f32),
                copy_of_target,
                self.speed,
            );
            return (Some(*next_order), Some(new_velocity));
        }
        (None, None)
    }

    // This method checks the current executed order for completion
    // If its completed, marks it as so, and processes results
    fn check_orders(&mut self, ent: &mut Ent) {
        if !ent.orders.is_empty() {
            let next_order = ent.orders.index(0);
            let mut did_complete_order = false;

            if !next_order.completed && next_order.executed {
                match next_order.order_type {
                    OrderType::Move | OrderType::AttackMove => {
                        // To complete either a move or attack move order, unit must reach it's destination
                        if self.has_arrived_at(ent, next_order.current_move_target) {
                            // Mark this order as completed
                            did_complete_order = true;

                            // The unit has moved to it's target successfully
                            // Clear it's velocity so it can rest
                            self.clear_velocity();
                        }
                    }
                    // A follow order can never be completed!
                    // It can only get cleard or canceled (if the followed unit dies)
                    // To complete an attack or lazy attack order, the target must be DEAD!
                    // To complete a mine order, ore must be destroyed (emptied)!
                    // That gets checked right before trying to attack it during execution,
                    // So it can get completed and cleaned up there as well.
                    // Same thing with HoldPosition, gets cleared right after execution.
                    OrderType::LazyAttack
                    | OrderType::Attack
                    | OrderType::Follow
                    | OrderType::HoldPosition
                    | OrderType::Mine => (),
                }
            }
            if did_complete_order {
                ent.orders.index_mut(0).complete();
            }
        }
    }

    // This method removes completed orders from the unit's order vector
    fn clear_completed_orders(&mut self, ent: &mut Ent) {
        ent.orders.retain(|order| !order.completed);
    }

    pub fn clear_all_but_current_order(&mut self, ent: &mut Ent) {
        if ent.orders.len() > 1 {
            ent.orders.drain(1..ent.orders.len());
        }
    }

    // This method executes a stop order to the unit
    // Stop order clears velocity, falsifies attack flag, and sets state to Stop
    // It also clears orders, meaning it removes itself. pretty handy.
    pub fn stop(&mut self, ent: &mut Ent) {
        // Cancel all orders
        ent.clear_orders();
        // Set state to stopped
        self.stop_attacking();
        ent.state = State::Stop;
        self.clear_velocity();
    }

    // This method executes a hold position order to the unit
    // Hold position order clears velocity, and sets state to Hold
    // It also clears orders, meaning it removes itself. pretty handy.
    fn hold_position(&mut self, ent: &mut Ent) {
        // Cancel all orders, but the hold position order so it gets cleared normally
        self.clear_all_but_current_order(ent);
        // Stop moving
        self.clear_velocity();
        // Set state to hold
        ent.state = State::Hold;
    }

    pub fn stop_attacking(&mut self) {
        self.is_attacking = false;
        self.attack_line_render_latch_point_delta = None;
    }

    pub fn start_attacking(&mut self, ent: &mut Ent, attack_target_rect: Rect) {
        self.clear_velocity();
        ent.state = State::Busy;
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

    pub fn has_target_in_hover_distance(&self, ent: &mut Ent, target: Vector2D<f32>) -> bool {
        let rect_center = ent.get_rect().center();
        (target - Vector2D::<f32>::new(rect_center.x as f32, rect_center.y as f32)).length()
            <= FOLLOW_ORDER_HOVER_DISTANCE
    }

    pub fn has_target_in_range_from_id(
        &self,
        ent: &mut Ent,
        world_info: &WorldInfo,
        target_id: EntID,
    ) -> (bool, f32) {
        let target_pos = world_info
            .get_ent_rect_center_poisition_by_id(target_id)
            .expect(">> Could not find attack target position from world info");
        let self_rect_center = ent.get_rect().center();
        let distance = (Vector2D::<f32>::new(self_rect_center.x as f32, self_rect_center.y as f32)
            - target_pos)
            .length();
        (distance <= self.range, distance)
    }

    pub fn has_target_in_range_from_rect_center(
        &self,
        ent: &mut Ent,
        target_position: Vector2D<f32>,
    ) -> (bool, f32) {
        let self_rect_center = ent.get_rect().center();
        let distance = (Vector2D::<f32>::new(self_rect_center.x as f32, self_rect_center.y as f32)
            - target_position)
            .length();
        (distance <= self.range, distance)
    }

    pub fn get_closest_target_in_range(
        &mut self,
        ent: &mut Ent,
        world_info: &mut WorldInfo,
    ) -> (bool, EntTarget, f32) {
        // Check if any other unit is in range; if so, issue attack order to the closest one
        let mut closest_ent_in_range = empty_ent_target();
        let mut has_target_in_range = false;
        let mut closest_ent_distance = self.range;
        for (ent_id, ent_rect_center) in &world_info.ent_rect_center {
            if *ent_id == ent.id {
                // Cannot target self; return early
                continue;
            }
            if world_info
                .get_ent_owner_by_id(*ent_id)
                .expect(">> Could not find ent team by id")
                == ent.owner
            {
                // Cannot targert an ent on the same team; return early
                continue;
            }

            let (is_in_range, distance) =
                self.has_target_in_range_from_rect_center(ent, *ent_rect_center);

            if !is_in_range {
                // Too far away; return early
                continue;
            }

            // Only return the closest possible target
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
                    ent_parent_type: world_info.get_ent_parent_type_by_id(*ent_id),
                };
            }
        }
        (
            has_target_in_range,
            closest_ent_in_range,
            closest_ent_distance,
        )
    }

    pub fn has_arrived_at(&self, ent: &mut Ent, target: Vector2D<f32>) -> bool {
        let rect_center = ent.get_rect().center();
        (target - Vector2D::<f32>::new(rect_center.x as f32, rect_center.y as f32)).length()
            <= MAX_MOVE_ORDER_ERROR
    }

    fn check_surroundings(&mut self, ent: &mut Ent, world_info: &mut WorldInfo) {
        // Checks for Alert & Hold state
        // An alert unit has no pending orders in its queue
        // It should actively seek to attack enemy units that appear in its range
        // Hold units behave similarly, but issue lazy attacks instead (won't chase)
        if vec![State::Alert, State::Hold].contains(&ent.state) {
            let (has_target_in_range, closest_ent_in_range, _) =
                self.get_closest_target_in_range(ent, world_info);
            if has_target_in_range {
                // Check if the target is an ore
                // In wich case attack move should NOT consider it (its attack move, not mine move... for now)
                if closest_ent_in_range.ent_parent_type == Some(EntParentType::Ore) {
                    // Indeed the case, return early;
                    return;
                }
                let ent_rect = closest_ent_in_range
                    .ent_rect
                    .expect(">> Could not find ent rect by id");
                let attack_order = Order::new(
                    if ent.state == State::Alert {
                        OrderType::Attack
                    } else {
                        OrderType::LazyAttack
                    },
                    Vector2D::<f32>::new(ent_rect.x as f32, ent_rect.y as f32),
                    closest_ent_in_range,
                );
                // Issue attack order to closest in-range target
                // Bump it so that it takes precedence over this attack move order
                ent.bump_order(attack_order);
            }
        }
    }

    fn execute_next_order(
        &mut self,
        ent: &mut Ent,
        next_order: Order,
        next_order_direction_option: Option<Vector2D<f32>>,
        world_info: &mut WorldInfo,
    ) -> bool {
        match next_order.order_type {
            OrderType::Move => {
                ent.state = State::Busy;
                self.stop_attacking();
                self.set_desired_velocity(next_order_direction_option.expect(
                    ">> Could not set unit velocity; current order did not produce a direction vector",
                ));
            }
            OrderType::Attack | OrderType::LazyAttack => {
                let possible_attack_target = &next_order.ent_target;
                if possible_attack_target.ent_id.is_none() {
                    // No more target, attack is done!
                    return self.cancel_attack_order(ent, next_order);
                }
                let attack_target_id = possible_attack_target
                    .ent_id
                    .expect(">> Could not find attack target id from current order");
                if !world_info.has_ent_by_id(attack_target_id) {
                    // No more target, attack is done!
                    return self.cancel_attack_order(ent, next_order);
                }
                if self
                    .has_target_in_range_from_id(ent, world_info, attack_target_id)
                    .0
                {
                    // If target is in range, check if already attacking
                    if self.is_attacking {
                        world_info.damage_ent(attack_target_id, self.damage * TIME_STEP);
                    } else {
                        // Else, start attacking
                        self.start_attacking(ent, possible_attack_target.ent_rect.expect(">> Could not get ent rect from attack target, but could find rect center position in world info?"));
                    }
                } else {
                    // If target is not in range, check order type
                    // If regular attack, chase
                    // If lazy attack, complete order
                    if next_order.order_type == OrderType::Attack {
                        // If normal attack move towards it
                        self.set_desired_velocity(next_order_direction_option.expect(">> Could not set unit velocity; current order did not produce a direction vector"));
                        // And mark as not attacking
                        self.stop_attacking()
                    } else {
                        return self.cancel_attack_order(ent, next_order);
                    }
                }
                ent.state = State::Busy;
            }
            OrderType::AttackMove => {
                ent.state = State::Alert;
                self.stop_attacking();
                self.set_desired_velocity(next_order_direction_option.expect(">> Could not set unit velocity; current order did not produce a direction vector"));
            }
            OrderType::Follow => {
                // Unit should stop moving if it gets within a certain distance of it's follow target
                // If target is dead, complete order
                if next_order.ent_target.ent_id.is_none() {
                    // No target, order completed!
                    // Return true for a completed order
                    return true;
                }
                let follow_target_id = next_order
                    .ent_target
                    .ent_id
                    .expect(">> Could not get entity id form follow target");
                if !world_info.has_ent_by_id(follow_target_id) {
                    // Target is dead!
                    // Return true for a completed order
                    return true;
                }
                if self.has_target_in_hover_distance(ent, next_order.current_move_target) {
                    self.clear_velocity();
                } else {
                    self.set_desired_velocity(next_order_direction_option.expect(
                        ">> Could not set unit velocity; current order did not produce a direction vector",
                    ));
                }
                self.stop_attacking();
                ent.state = State::Busy;
            }
            OrderType::HoldPosition => {
                self.hold_position(ent);
                return true;
            }
            OrderType::Mine => {
                // TODO: Add actual mining, for now it will just follow the ore
                //      => 1. Damage the ore with a special unit type dependant penalty to the unit dmg (miner will have no penalty, but low dmg)
                //      => 2. After a certain amount of dmg, dependant on ore patch density, some ore will be dropped (TODO: rename ore.rs to ore_patch.rs?)
                //      => 3. Unit mining stores this ore on itself (maybe up to a certain carry_capacity, and with some increase to its mass  so its more clumsy (?)
                //                                                  or just a speed debuff for carrying (maybe no attacking att full cap, must drop to attack?))
                //      => 4. Unit must beam ore back into mainframe for collection and later use
                // Unit should stop moving if it gets within a certain distance of it's mine target
                // If target is no longer, complete order
                if next_order.ent_target.ent_id.is_none() {
                    // No target, order completed!
                    // Return true for a completed order
                    return true;
                }
                let mine_target_id = next_order
                    .ent_target
                    .ent_id
                    .expect(">> Could not get entity id form follow target");
                if !world_info.has_ent_by_id(mine_target_id) {
                    // Target is dead!
                    // Return true for a completed order
                    return true;
                }
                if self
                    .has_target_in_range_from_rect_center(ent, next_order.current_move_target)
                    .0
                {
                    self.clear_velocity();
                } else {
                    self.set_desired_velocity(next_order_direction_option.expect(
                        ">> Could not set unit velocity; current order did not produce a direction vector",
                    ));
                }
                self.stop_attacking();
                ent.state = State::Busy;
            }
        }
        false
    }

    fn cancel_attack_order(&mut self, ent: &mut Ent, next_order: Order) -> bool {
        // Check if this is actualy an attack order
        // Note: If new attack order types are added, this vec macro needs updating...
        if !vec![OrderType::Attack, OrderType::LazyAttack].contains(&next_order.order_type) {
            // Return false for an uncompleted order
            return false;
        }
        // Mark self as not attacking
        self.stop_attacking();
        // If regular attack: Clear velocity; Attack order could have a unit moving
        // If lazy attack: set state to hold
        match next_order.order_type {
            OrderType::Attack => self.clear_velocity(),
            OrderType::LazyAttack => ent.state = State::Hold,
            _ => (),
        }
        // Return true for a completed order
        true
    }

    fn set_desired_velocity(&mut self, target: Vector2D<f32>) {
        self.desired_velocity = target;
    }

    fn apply_steering(&mut self) {
        if self.velocity != self.desired_velocity {
            let steering = self.desired_velocity - self.velocity;
            self.velocity += steering / self.mass;
        }
    }
}
