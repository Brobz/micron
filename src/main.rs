// Initial code from: https://github.com/amengede/getIntoGameDev/blob/main/Rust/
// (for early rust && SDL2)

// TODO:
//          ??. Limit framerate somehow?
//          ??. Fix zoom out jankiness (would like it for the zoom behaviour to be reversed when zooming out... why is this so hard)
//          1. Add attack move order
//          2. Figure out proper combat (attack speed (maybe not?))
//          3. Add nice beam animation to current attack (several small boxes or circles travelling from one end of the line to the other)
//          ??. Add some logic to allow a unit to move while attacking (would need some sort of anchor target system; maintain target while in range, lose it when out of range)
//          ??. Add stop order (S) [stop order + attack order = nice combo (need to figure out atack move first)]
//          ??. Add patrol order (R)

mod consts;
mod structs;

use rand::Rng;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::BlendMode;
use structs::camera::Camera;
use structs::ent::Ent;
use structs::input::Input;
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
    let clear_color = Color::RGB(64, 192, 255);

    let mut event_queue = sdl_context.event_pump().unwrap();

    let mut world = World::new();
    let mut world_info = WorldInfo::new();
    let mut camera = Camera::new();
    let mut rng = rand::thread_rng();

    for _ in 1..10 {
        let new_ent = Ent::new(
            100,
            Vector2D::<f32>::new(
                rng.gen_range(MAP_WIDTH / 2 + 25..MAP_WIDTH / 2 + SCREEN_WIDTH) as f32,
                rng.gen_range(MAP_HEIGHT / 2 + 25..MAP_HEIGHT / 2 + SCREEN_HEIGHT) as f32,
            ),
            Point::new(rng.gen_range(5..50), rng.gen_range(5..50)),
        );
        world_info.add_ent(&new_ent);
        world.units.push(Unit::new(new_ent));
    }

    loop {
        //////////////////////// USER INPUT /////////////////////////

        if !Input::process_input(&mut event_queue, &mut camera, &mut world) {
            break;
        }

        //////////////////////// UPDATE GAME STATE /////////////////////////

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

        //////////////////////// RENDER GAME STATE /////////////////////////

        // Clear screen
        canvas.set_draw_color(clear_color);
        canvas.set_scale(camera.scale.x, camera.scale.y).ok();

        // Set viewport to cover whole map
        canvas.set_viewport(Rect::new(
            0 - MAP_PADDING as i32,
            0 - MAP_PADDING as i32,
            MAP_WIDTH + MAP_PADDING * 2,
            MAP_HEIGHT + MAP_PADDING * 2,
        ));

        // Clear it
        canvas.fill_rect(camera.get_scaled_screen_area()).ok();

        // Set viewport back to where the camera is
        canvas.set_viewport(Rect::new(
            camera.position.x,
            camera.position.y,
            canvas.viewport().width(),
            canvas.viewport().height(),
        ));

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
