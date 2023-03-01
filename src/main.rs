// Initial code from: https://github.com/amengede/getIntoGameDev/blob/main/Rust/
// (for early rust && SDL2)

// TODO:
//  Some important backlog stuff
//  ??. Change Selection struct bools to a enum State type of deal (check clipy::pedantic)
//  ??. Limit framerate somehow (try using sdl2_timing)?
//  ??. Figure out a way to only draw required orders (i.e. a selection of units gets shift moved around; all but the line from unit to first waypoint will be redrawn uselessly)
//      ==> This MASSIVELY boosts performace, not drawing orders for 1k units eliminates all lag when queuing. this would effectively cut 90% of the orders to draw out

// Some current stuff

//  1. Get creative with structures
//      0. Figure out central structure (tower defense "nexus", what can it do, how does it get attacked, how does the player win the game, etc.)
//      1. Add it, abstracting as much as possible

//  2. Test collision feel & benchmark
//      0. Maybe try a Mutalisk style thing - can overlap freely while moving, but slowly unbunch until completely separated when resting
//      1. If not, will definitely need to implement pathfinding (could give A* a try)

//  3. Refactor game system
//      0. Change all pair data types on structs to Vector2D<f32>; Then convert back to point as needed for drawing (might be better then current way of things)

//  4. Get creative with combat
//      0. Figure out proper combat (attack speed (maybe not? check next list #))
//      1. Add nice beam animation to current attack (several small boxes or circles travelling from one end of the line to the other)

//  Some less important backlog stuff
//  ??. Add some logic to allow a unit to move while attacking (would need some sort of anchor target system; maintain target while in range, lose it when out of range)
//  ??. Add patrol order (R) ?
//  ??. Fix zoom out jankiness (would like it for the zoom behaviour to be reversed when zooming out...)

mod consts;
mod structs;

use consts::values::{SCREEN_HEIGHT, SCREEN_WIDTH};
use structs::camera::Camera;

use structs::input::Input;
use structs::world_info::WorldInfo;

use crate::{consts::*, structs::*};

use structs::world::*;

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

    spawn_debug_ents(500, &mut world, &mut world_info);

    loop {
        //////////////////////// USER INPUT /////////////////////////

        // Process player input
        // If this method returns false, the window was closed; exit loop
        if !Input::process_input(&mut event_queue, &mut camera, &mut world, &mut world_info) {
            break;
        }

        //////////////////////// UPDATE GAME STATE /////////////////////////

        // Tick World
        world.tick(&mut world_info);

        //////////////////////// RENDER GAME STATE /////////////////////////

        // Draw World
        world.draw(&mut canvas, &mut world_info, &mut camera);

        // Refresh screen
        canvas.present();
    }

    Ok(())
}
