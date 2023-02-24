use std::ops::Index;
use std::ops::IndexMut;

use vector2d::Vector2D;

use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::consts::helper::get_direction_from_to;
use crate::ent::Ent;
use crate::TIME_STEP;

use crate::order::Order;
use crate::order::OrderType;

pub struct Unit {
    pub ent: Ent,
    pub speed: f32,
    selected: bool,
    velocity: Vector2D<f32>,
    pub orders: Vec<Order>,
}

impl Unit {
    pub fn new(ent: Ent, speed: f32, velocity: Vector2D<f32>) -> Unit {
        Unit {
            ent,
            speed,
            selected: false,
            velocity,
            orders: Vec::<Order>::new(),
        }
    }

    pub fn tick(&mut self) {
        self.apply_velocity();
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        // Draw order waypoints, if selected
        if self.selected {
            canvas.set_draw_color(Color::RGB(0, 150, 0));
            for (i, order) in self.orders.iter().enumerate() {
                // Draw lines connecting order waypoints
                match i {
                    // If this is the next order, draw  a line from unit to waypoint
                    0 => {
                        canvas
                            .draw_line(
                                self.ent.get_rect().center(),
                                Point::new(order.target.x as i32, order.target.y as i32),
                            )
                            .ok()
                            .unwrap_or_default();
                    }
                    // Else, draw line from last waypoint to this one
                    _ => {
                        let previous_order_target = self.orders.index(i - 1).target;
                        canvas
                            .draw_line(
                                Point::new(
                                    previous_order_target.x as i32,
                                    previous_order_target.y as i32,
                                ),
                                Point::new(order.target.x as i32, order.target.y as i32),
                            )
                            .ok()
                            .unwrap_or_default();
                    }
                }
                let waypoint_rect: Rect = Rect::from_center(
                    Point::new(order.target.x as i32, order.target.y as i32),
                    5,
                    5,
                );
                canvas.fill_rect(waypoint_rect).ok().unwrap_or_default();
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

        // Draw self
        canvas.set_draw_color(Color::RGB(50, 10, 50));
        let rect: Rect = Rect::new(
            self.ent.position.x as i32,
            self.ent.position.y as i32,
            self.ent.rect_size.x as u32,
            self.ent.rect_size.y as u32,
        );

        canvas.fill_rect(rect).ok().unwrap_or_default();
    }

    pub fn set_velocity(&mut self, _velocity: Vector2D<f32>) {
        self.velocity = _velocity;
    }

    pub fn clear_velocity(&mut self) {
        self.velocity = Vector2D::<f32>::new(0.0, 0.0);
    }

    // This method applies velocity each tick to the unit
    pub fn apply_velocity(&mut self) {
        self.ent.position.x += self.velocity.x * TIME_STEP;
        self.ent.position.y += self.velocity.y * TIME_STEP;
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
            let copy_of_target = next_order.target;
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
    pub fn update_orders(&mut self) {
        if self.orders.len() > 0 {
            let next_order = self.orders.index_mut(0);

            if !next_order.completed && next_order.executed {
                match next_order.order_type {
                    OrderType::Move => {
                        let rect_center = self.ent.get_rect().center();
                        if (next_order.target
                            - Vector2D::<f32>::new(rect_center.x as f32, rect_center.y as f32))
                        .length()
                            <= 3.0
                        {
                            // Mark this order as completed
                            next_order.complete();

                            // The unit has moved to it's target successfully
                            // If this was the last action in the queue, clear it's acceleration so it can rest
                            if self.orders.len() == 1 {
                                self.clear_velocity();
                            };
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
