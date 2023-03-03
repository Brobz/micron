use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton, rect::Point, EventPump};
use vector2d::Vector2D;

use crate::{
    consts::{
        debug_flags::DEBUG_CAN_CONTROL_CPU,
        helper::{empty_ent_target, select_all_army},
    },
    enums::{game_object::GameObject, unit_type::UnitType},
};

use super::{
    camera::Camera,
    ent::{EntParentType, Owner},
    order::{EntTarget, Order, OrderType},
    selection::MouseCommand,
    world::World,
};

// TODO: CLEANUP THIS FILE

pub struct Input;

impl Input {
    pub fn process_input(
        event_queue: &mut EventPump,
        camera: &mut Camera,
        world: &mut World,
    ) -> bool {
        for event in event_queue.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return false;
                }

                Event::MouseWheel { direction, y, .. } => {
                    camera.zoom(direction, y);
                }

                Event::MouseMotion { x, y, .. } => {
                    Self::process_mouse_motion(x, y, camera, world);
                }

                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    Self::process_mouse_button_down(mouse_btn, x, y, camera, world);
                }
                Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => {
                    Self::process_mouse_button_up(mouse_btn, x, y, camera, world);
                }

                Event::KeyDown {
                    keycode: Some(Keycode::LShift),
                    ..
                } => world.selection.shift_press(),

                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => world.selection.clear(&mut world.game_objects),

                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => world.selection.engange_command(MouseCommand::Action),

                Event::KeyUp {
                    keycode: Some(Keycode::LShift),
                    ..
                } => world.selection.shift_release(),

                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    for game_object in &mut world.game_objects {
                        match game_object {
                            GameObject::Unit(ent, unit) => {
                                if ent.selected()
                                    && (ent.owner == Owner::Player || DEBUG_CAN_CONTROL_CPU)
                                {
                                    // Issue stop order to owned selected units
                                    match unit {
                                        UnitType::Scout(unit)
                                        | UnitType::Miner(unit)
                                        | UnitType::Collector(unit) => unit.stop(ent),
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }

                Event::KeyDown {
                    keycode: Some(Keycode::H),
                    ..
                } => {
                    for game_object in &mut world.game_objects {
                        match game_object {
                            GameObject::Unit(ent, _) => {
                                if ent.selected()
                                    && (ent.owner == Owner::Player || DEBUG_CAN_CONTROL_CPU)
                                {
                                    // If queueing, need to figure out if this is the first order of the chain or not
                                    // To know what to render
                                    let mut hold_position_spot: Option<Vector2D<f32>> = None;
                                    if world.selection.queueing {
                                        if let Some(order) = ent.orders.last() {
                                            hold_position_spot = Some(order.current_move_target);
                                        } else {
                                            hold_position_spot = Some(ent.position);
                                        }
                                    }
                                    // Issue hold position order to owned selected units
                                    let hold_position_order = Order::new(
                                        OrderType::HoldPosition,
                                        hold_position_spot.unwrap_or(ent.position),
                                        empty_ent_target(),
                                    );
                                    ent.add_order(hold_position_order, !world.selection.queueing);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                Event::KeyDown {
                    keycode: Some(Keycode::F2),
                    ..
                } => {
                    select_all_army(world);
                }

                _ => {}
            }
        }
        true
    }

    fn process_mouse_button_up(
        mouse_btn: MouseButton,
        x: i32,
        y: i32,
        camera: &mut Camera,
        world: &mut World,
    ) {
        camera.update_mouse_rect(Point::new(x, y));
        let scaled_mouse_pos = camera.get_scaled_mouse_pos();
        match mouse_btn {
            MouseButton::Left => match world.selection.left_click_command {
                MouseCommand::Select => world
                    .selection
                    .close(scaled_mouse_pos, &mut world.game_objects),
                MouseCommand::Action => world.selection.release_command(),
            },
            MouseButton::Middle => camera.release(),
            MouseButton::Right | MouseButton::X1 | MouseButton::X2 | MouseButton::Unknown => (),
        }
    }

    fn process_mouse_button_down(
        mouse_btn: MouseButton,
        x: i32,
        y: i32,
        camera: &mut Camera,
        world: &mut World,
    ) {
        // First, update mouse position
        camera.update_mouse_rect(Point::new(x, y));
        // Then, get scaled mouse position
        let scaled_mouse_pos = camera.get_scaled_mouse_pos();
        // This right click might issue an attack, mine or follow order, we will need to store its possible target
        let mut click_target = empty_ent_target();
        // Flag to know if we found at least one target
        let mut found_target = false;

        // Check wether we clicked on something attackable
        for game_object in &world.game_objects {
            match game_object {
                GameObject::Unit(ent, _)
                | GameObject::Structure(ent, _)
                | GameObject::OrePatch(ent, _)
                | GameObject::Ore(ent, _) => {
                    if ent
                        .get_rect()
                        .has_intersection(camera.get_scaled_mouse_rect())
                    {
                        click_target = EntTarget {
                            ent_id: Some(ent.id),
                            ent_rect: Some(ent.get_rect()),
                            ent_owner: Some(ent.owner),
                            ent_parent_type: Some(ent.parent_type()),
                        };
                        found_target = true;
                        break;
                    }
                }
            }
        }

        match mouse_btn {
            MouseButton::Left => {
                // Left click
                match world.selection.left_click_command {
                    MouseCommand::Select => {
                        // No command engaged, just open selection
                        world
                            .selection
                            .open(scaled_mouse_pos, &mut world.game_objects);
                    }
                    MouseCommand::Action => {
                        // Attack command engage
                        // This could either trigger a direct action or an action move
                        let target_is_ore_patch =
                            click_target.ent_parent_type == Some(EntParentType::OrePatch);
                        let target_is_ore =
                            click_target.ent_parent_type == Some(EntParentType::Ore);
                        for game_object in &mut world.game_objects {
                            match game_object {
                                GameObject::Unit(ent, unit_type) => {
                                    if ent.selected()
                                        && (ent.owner == Owner::Player || DEBUG_CAN_CONTROL_CPU)
                                    {
                                        if !found_target {
                                            // No action target found; Issue attack move order
                                            let action_move_order = Order::new(
                                                OrderType::ActionMove,
                                                Vector2D::<f32>::new(
                                                    scaled_mouse_pos.x as f32,
                                                    scaled_mouse_pos.y as f32,
                                                ),
                                                empty_ent_target(),
                                            );
                                            ent.add_order(
                                                action_move_order,
                                                !world.selection.queueing,
                                            );
                                        } else {
                                            // Action target found; check if it is a valid one
                                            // (defaults to self in case it's not there, canceling the action (it should be there tho))
                                            let action_target_id =
                                                click_target.ent_id.unwrap_or(ent.id);
                                            if action_target_id == ent.id {
                                                // Cannot action yourself.. for now!
                                                continue;
                                            }

                                            if let Some(ent_owner) = click_target.ent_owner {
                                                if ent_owner == ent.owner {
                                                    // Cannot attack an ent on the same team!
                                                    continue;
                                                }
                                            }

                                            // Now we know we will either attack, mine or collect from this target!
                                            // Check ent parent type to know what to do
                                            let new_order_type = if target_is_ore_patch {
                                                OrderType::Mine
                                            } else if target_is_ore {
                                                OrderType::Collect
                                            } else {
                                                OrderType::Attack
                                            };

                                            // Check if the unit can actually perform this action
                                            match unit_type {
                                                UnitType::Scout(_) => {
                                                    if new_order_type != OrderType::Attack {
                                                        continue;
                                                    }
                                                }
                                                UnitType::Miner(_) => {
                                                    if new_order_type != OrderType::Mine {
                                                        continue;
                                                    }
                                                }
                                                UnitType::Collector(_) => {
                                                    if new_order_type != OrderType::Collect {
                                                        continue;
                                                    }
                                                }
                                            }

                                            let new_order = Order::new(
                                                new_order_type,
                                                Vector2D::<f32>::new(
                                                    scaled_mouse_pos.x as f32,
                                                    scaled_mouse_pos.y as f32,
                                                ),
                                                EntTarget {
                                                    ent_id: Some(action_target_id),
                                                    ent_rect: click_target.ent_rect,
                                                    ent_owner: click_target.ent_owner,
                                                    ent_parent_type: click_target.ent_parent_type,
                                                },
                                            );
                                            ent.add_order(new_order, !world.selection.queueing);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            MouseButton::Middle => {
                camera.grab(&scaled_mouse_pos);
            }
            MouseButton::Right => {
                // Right click
                // Issue either a move or attack command

                // Release left click command (if any)
                world.selection.release_command();
                // Check wether we clicked on something attackable, or minable
                for game_object in &world.game_objects {
                    match game_object {
                        GameObject::Unit(ent, _)
                        | GameObject::Structure(ent, _)
                        | GameObject::OrePatch(ent, _)
                        | GameObject::Ore(ent, _) => {
                            if ent
                                .get_rect()
                                .has_intersection(camera.get_scaled_mouse_rect())
                            {
                                click_target = EntTarget {
                                    ent_id: Some(ent.id),
                                    ent_rect: Some(ent.get_rect()),
                                    ent_owner: Some(ent.owner),
                                    ent_parent_type: Some(ent.parent_type()),
                                };
                                break;
                            }
                        }
                    }
                }

                let target_is_ore_patch =
                    click_target.ent_parent_type == Some(EntParentType::OrePatch);
                let target_is_ore = click_target.ent_parent_type == Some(EntParentType::Ore);

                for game_object in &mut world.game_objects {
                    match game_object {
                        GameObject::Unit(ent, unit_type) => {
                            if ent.selected()
                                && (ent.owner == Owner::Player || DEBUG_CAN_CONTROL_CPU)
                            {
                                if !found_target {
                                    // No right click target found; Issue move order
                                    let move_order = Order::new(
                                        OrderType::Move,
                                        Vector2D::<f32>::new(
                                            scaled_mouse_pos.x as f32,
                                            scaled_mouse_pos.y as f32,
                                        ),
                                        empty_ent_target(),
                                    );
                                    ent.add_order(move_order, !world.selection.queueing);
                                } else {
                                    // Right click arget found; check if it is a valid one
                                    // (defaults to self in case it's not there, canceling the attack (it should be there tho))
                                    let attack_target_id = click_target.ent_id.unwrap_or(ent.id);
                                    if attack_target_id == ent.id {
                                        // Cannot attack yourself!
                                        continue;
                                    }
                                    // At this point, we know we will either attack, follow, collect or mine this target, depending on it's type and team
                                    // Now we know we will either attack, mine or collect from this target!
                                    // Check ent parent type to know what to do
                                    let new_order_type = if target_is_ore_patch {
                                        OrderType::Mine
                                    } else if target_is_ore {
                                        OrderType::Collect
                                    } else if click_target.ent_owner == Some(ent.owner) {
                                        OrderType::Follow
                                    } else {
                                        OrderType::Attack
                                    };

                                    // Check if the unit can actually perform this action
                                    match unit_type {
                                        UnitType::Scout(_) => {
                                            if !(vec![OrderType::Attack, OrderType::Follow]
                                                .contains(&new_order_type))
                                            {
                                                continue;
                                            }
                                        }
                                        UnitType::Miner(_) => {
                                            if !(vec![OrderType::Mine, OrderType::Follow]
                                                .contains(&new_order_type))
                                            {
                                                continue;
                                            }
                                        }
                                        UnitType::Collector(_) => {
                                            if !(vec![OrderType::Collect, OrderType::Follow]
                                                .contains(&new_order_type))
                                            {
                                                continue;
                                            }
                                        }
                                    }

                                    let new_order = Order::new(
                                        new_order_type,
                                        Vector2D::<f32>::new(
                                            scaled_mouse_pos.x as f32,
                                            scaled_mouse_pos.y as f32,
                                        ),
                                        EntTarget {
                                            ent_id: Option::Some(attack_target_id),
                                            ent_rect: click_target.ent_rect,
                                            ent_owner: click_target.ent_owner,
                                            ent_parent_type: click_target.ent_parent_type,
                                        },
                                    );
                                    ent.add_order(new_order, !world.selection.queueing);
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
            MouseButton::Unknown | MouseButton::X1 | MouseButton::X2 => (),
        }
    }

    fn process_mouse_motion(x: i32, y: i32, camera: &mut Camera, world: &mut World) {
        camera.update_mouse_rect(Point::new(x, y));
        let scaled_mouse_pos = camera.get_scaled_mouse_pos();
        world
            .selection
            .tick(scaled_mouse_pos, &mut world.game_objects);
        if camera.is_anchored() {
            let anchored_mouse_pos = camera.get_anchored_mouse_pos();
            camera.drag_to(anchored_mouse_pos);
        }
    }
}
