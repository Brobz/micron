// Initial code from: https://github.com/bevyengine/bevy/blob/latest/examples/games/breakout.rs
// (for early rust && bevy refs)

// TODO:
//          1. Move Velocity inside of Ent and Unit
//          2. Add order queue and completion / cancelation of orders (stopping when H is pressed or when reached destination)
//          3. Add shift clicking for adding into queue, regular clicking replaces queue (cancels all)

mod consts;
mod structs;

use bevy::{prelude::*, sprite::collide_aabb::collide, time::FixedTimestep};

use crate::{consts::*, structs::*};

use crate::mouse_info::*;
use crate::selection::*;

use crate::collider::*;
use crate::ent::*;
use crate::unit::*;

use crate::setup::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: SCREEN_WIDTH,
                height: SCREEN_HEIGHT,
                title: "Window Resize Example".to_string(),
                ..default()
            },
            ..default()
        }))
        .insert_resource(MouseInfo {
            position: Vec2::from([-1f32, -1f32]),
            left_button: false,
            right_button: false,
        })
        .insert_resource(Selection {
            open: false,
            just_closed: false,
            origin: Vec2::new(-1f32, -1f32),
            center: Vec3::new(-1f32, -1f32, -1f32),
            scale: Vec3::new(0f32, 0f32, 0f32),
            current: Vec::new(),
        })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(mouse_button_input)
                .with_system(get_cursor_position.before(mouse_button_input))
                .with_system(update_debug_info.after(mouse_button_input))
                .with_system(draw_selection_box.after(mouse_button_input))
                .with_system(check_for_selection_box_collisions.after(draw_selection_box))
                .with_system(apply_velocity.before(check_for_selection_box_collisions))
                .with_system(issue_commands.after(check_for_selection_box_collisions)),
        )
        .add_system(bevy::window::close_on_esc)
        .run();
}
