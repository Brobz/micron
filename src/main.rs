// Initial code from: https://github.com/amengede/getIntoGameDev/blob/main/Rust/
// (for early rust && SDL2)

// TODO:
//          1. Add nice beam animation to current attack (several small boxes or circles travelling from one end of the line to the other)
//          2. Figure out proper combat (attack speed (maybe not?), attack_move)
//          ??. Add some logic to allow a unit to move while attacking (would need some sort of anchor target system; maintain target while in range, lose it when out of range)
//          ??. Add stop order (S)
//          ??. Add patrol order (R)

mod consts;
mod structs;

use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::BlendMode;
use structs::ent::{Ent, EntID};
use structs::order::{Order, OrderType};
use structs::world_info::WorldInfo;
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
    let mut world_info = WorldInfo::new();

    let mut rng = rand::thread_rng();

    for _ in 1..10 {
        let new_ent = Ent::new(
            100,
            Vector2D::<f32>::new(
                rng.gen_range(0..SCREEN_WIDTH) as f32,
                rng.gen_range(0..SCREEN_HEIGHT) as f32,
            ),
            Point::new(rng.gen_range(1..50), rng.gen_range(1..50)),
        );
        world_info.add_ent(&new_ent);
        world.units.push(Unit::new(new_ent));
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
                    let mut possible_attack_target: Option<EntID> = None;
                    if mouse_btn.eq(&MouseButton::Left) {
                        // Left click
                        // Open selection
                        world.selection.open(Point::new(x, y), &mut world.units);
                    } else if mouse_btn.eq(&MouseButton::Right) {
                        // Right click
                        // Issue either a move or attack command
                        let mouse_rect = Rect::new(x, y, 2, 2);

                        // Check wether we clicked on something attackable
                        for unit in world.units.iter() {
                            if unit.ent.get_rect().has_intersection(mouse_rect) {
                                possible_attack_target = Option::Some(unit.ent.id);
                                break;
                            }
                        }

                        for unit in world.units.iter_mut() {
                            if unit.selected() {
                                if possible_attack_target.is_none() {
                                    // No attack target found; Issue move order
                                    let move_order = Order::new(
                                        OrderType::Move,
                                        Vector2D::<f32>::new(x as f32, y as f32),
                                        None,
                                    );
                                    unit.add_order(move_order, !world.selection.queueing);
                                } else {
                                    // Attack target found; check if it is a valid one
                                    // (defaults to self in case it's not there, canceling the attack (it should be there tho))
                                    let attack_target =
                                        possible_attack_target.unwrap_or(unit.ent.id);
                                    if attack_target == unit.ent.id {
                                        // Cannot attack yourself!
                                        continue;
                                    }
                                    let attack_order = Order::new(
                                        OrderType::Attack,
                                        Vector2D::<f32>::new(x as f32, y as f32),
                                        Option::Some(attack_target),
                                    );
                                    unit.add_order(attack_order, !world.selection.queueing);
                                }
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

        // Tick orders
        for unit in world.units.iter_mut() {
            for order in unit.orders.iter_mut() {
                if order.attack_target.is_none() {
                    continue;
                }
                // For every attack order, update it's target position to the attacked entities position
                let possible_target_position =
                    world_info.get_ent_poisition_by_id(&order.attack_target.unwrap());

                if let Some(target_position) = possible_target_position {
                    order.move_target = target_position;
                }
            }
        }

        // Tick units
        // Also, store the index of any units that are to be removed after this tick
        let mut ent_cleanup_list: Vec<usize> = Vec::<usize>::new();
        for (i, unit) in world.units.iter_mut().enumerate() {
            // Check if this unit's entity still exists in the world
            if world_info.has_ent(&unit.ent) {
                // If so, tick and update world_info
                unit.tick(&mut world_info);
                world_info.update_ent(&unit.ent);
            } else {
                // If not, add to cleanup list
                ent_cleanup_list.push(i);
            }
        }

        // Remove dead units
        for i in ent_cleanup_list.iter() {
            world.units.remove(*i);
        }

        // DRAW

        // Clear screen
        canvas.set_draw_color(clear_color);
        canvas.fill_rect(screen_area).ok();

        // Draw units
        for unit in world.units.iter() {
            unit.draw(&mut canvas);
        }

        // Draw Health Bars
        world_info.draw_health_bars(&mut canvas);

        // Draw selection box
        world.selection.draw(&mut canvas);

        // Refresh screen
        canvas.present();
    }

    Ok(())
}
