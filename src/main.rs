// Initial code from: https://github.com/amengede/getIntoGameDev/blob/main/Rust/
// (for early rust && SDL2)

// TODO:
//          ??. Limit framerate somehow (try using sdl2_timing)?
//          0. Add "team" property to Ent; Only allow player to issue orders to Player team ents, disallow combat between same team ents
//          1. Have a GameObject Enum that can be either Unit or Structure; ent will be contained inside those; refactor all world.units calculations to use world.game_objects
//          2. Change all pair data types on structs to Vector2D<f32>; Then convert back to point as needed for drawing (might be better then current way of things)
//          3. Figure out proper combat (attack speed (maybe not? check next list #))
//          4. Add nice beam animation to current attack (several small boxes or circles travelling from one end of the line to the other)
//          ??. Add some logic to allow a unit to move while attacking (would need some sort of anchor target system; maintain target while in range, lose it when out of range)
//          ??. Add stop order (S) [stop order + attack order = nice combo (need to figure out atack move first)]
//          ??. Add patrol order (R)
//          ??. Fix zoom out jankiness (would like it for the zoom behaviour to be reversed when zooming out... why is this so hard)

mod consts;
mod structs;

use consts::values::SCREEN_BACKGROUND_COLOR;
use consts::values::{MAP_HEIGHT, MAP_PADDING, MAP_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl2::rect::Rect;
use structs::camera::Camera;

use structs::ent::EntID;
use structs::input::Input;
use structs::world_info::WorldInfo;

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
        .expect(">> Could not load window");

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .expect(">> Could not build canvas from window");

    let mut event_queue = sdl_context
        .event_pump()
        .expect(">> Coult not instantiate event_queue");

    let mut world = World::new();
    let mut world_info = WorldInfo::new();
    let mut camera = Camera::new();

    spawn_debug_ents(&mut world, &mut world_info);

    loop {
        //////////////////////// USER INPUT /////////////////////////

        if !Input::process_input(&mut event_queue, &mut camera, &mut world, &mut world_info) {
            break;
        }

        //////////////////////// UPDATE GAME STATE /////////////////////////

        // Tick units
        // Also, store the index of any units that are to be removed after this tick
        let mut ent_cleanup_list: Vec<EntID> = Vec::<EntID>::new();
        for unit in &mut world.units {
            // Check if this unit's entity still exists in the world
            if world_info.has_ent(&unit.ent) {
                // If so, tick and update world_info
                unit.tick(&mut world_info);
                world_info.update_ent(&unit.ent);
            } else {
                // If not, add to cleanup list
                ent_cleanup_list.push(unit.ent.id);
            }
        }

        // Remove dead units
        world
            .units
            .retain(|unit| !ent_cleanup_list.contains(&unit.ent.id));

        // Tick orders
        for unit in &mut world.units {
            for order in &mut unit.orders {
                if order.attack_target.ent_id.is_none() {
                    continue;
                }
                let attack_target_id = order
                    .attack_target
                    .ent_id
                    .expect(">> Could not find attack target id for order");
                // For every attack order, update it's target position to the attacked entities position
                let possible_target_position = world_info.get_ent_poisition_by_id(attack_target_id);

                if let Some(target_position) = possible_target_position {
                    order.move_target = target_position;
                }

                // Also update the attack target rect
                order.attack_target.ent_rect = world_info.get_ent_rect_by_id(attack_target_id);
            }
        }

        //////////////////////// RENDER GAME STATE /////////////////////////

        // Clear screen
        canvas.set_draw_color(SCREEN_BACKGROUND_COLOR);
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

        // Draw unit orders
        for unit in &world.units {
            unit.draw_orders(&mut canvas);
        }

        // Draw units
        for unit in &world.units {
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
