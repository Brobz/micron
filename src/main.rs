// Initial code from: https://github.com/amengede/getIntoGameDev/blob/main/Rust/
// (for early rust && SDL2)

// TODO:
//          1. Figure out combat (attack & attack move orders)
//          ??. Add stop order (H)

mod consts;
mod structs;

use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::BlendMode;
use structs::ent::Ent;
use structs::order::{Order, OrderType};
use vector2d::Vector2D;

use crate::{consts::*, structs::*};

use structs::world::*;

use crate::unit::*;

use crate::setup::*;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("micron!", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_blend_mode(BlendMode::Blend);
    let screen_area = Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
    let clear_color = Color::RGB(64, 192, 255);

    let mut running = true;
    let mut event_queue = sdl_context.event_pump().unwrap();

    let mut world = World::new();

    let mut rng = rand::thread_rng();

    for _ in 1..100 {
        world.units.push(Unit::new(
            Ent::new(
                100.0,
                Vector2D::<f32>::new(
                    rng.gen_range(0..SCREEN_WIDTH) as f32,
                    rng.gen_range(0..SCREEN_HEIGHT) as f32,
                ),
                Point::new(rng.gen_range(1..50) as i32, rng.gen_range(1..50) as i32),
            ),
            BASE_UNIT_SPEED,
            Vector2D::<f32>::new(
                rng.gen_range(-BASE_UNIT_SPEED..BASE_UNIT_SPEED),
                rng.gen_range(-BASE_UNIT_SPEED..BASE_UNIT_SPEED),
            ),
        ));
    }

    while running {
        for event in event_queue.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    running = false;
                }

                Event::MouseMotion { x, y, .. } => {
                    world.selection.tick(Point::new(x, y), &mut world.units);
                }

                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    if mouse_btn.eq(&MouseButton::Left) {
                        world.selection.open(Point::new(x, y), &mut world.units);
                    } else if mouse_btn.eq(&MouseButton::Right) {
                        for unit in world.units.iter_mut() {
                            if unit.selected() {
                                let move_order = Order::new(
                                    OrderType::Move,
                                    Vector2D::<f32>::new(x as f32, y as f32),
                                );
                                unit.add_order(move_order, !world.selection.queueing);
                            }
                        }
                    }
                }
                Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => {
                    if mouse_btn.eq(&MouseButton::Left) {
                        world.selection.close(Point::new(x, y), &mut world.units);
                    }
                }

                Event::KeyDown { keycode, .. } => {
                    if keycode.is_some() && keycode.unwrap() == Keycode::LShift {
                        world.selection.shift_press();
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if keycode.is_some() && keycode.unwrap() == Keycode::LShift {
                        world.selection.shift_release();
                    }
                }
                _ => {}
            }
        }

        // UPDATE

        // Execute orders
        for unit in world.units.iter_mut() {
            let (next_order_option, next_order_direction_option) = unit.execute_next_order();
            if next_order_option.is_some() {
                let next_order = next_order_option.unwrap();
                match next_order.order_type {
                    OrderType::Move => unit.set_velocity(next_order_direction_option.unwrap()),
                }
            }
            unit.update_orders();
        }

        // Tick units
        for unit in world.units.iter_mut() {
            unit.tick();
        }

        // DRAW

        // Clear screen
        canvas.set_draw_color(clear_color);
        canvas.fill_rect(screen_area).ok();

        // Draw units
        for unit in world.units.iter() {
            unit.draw(&mut canvas);
        }

        // Draw selection box
        world.selection.draw(&mut canvas);

        // Refresh screen
        canvas.present();
    }

    Ok(())
}
