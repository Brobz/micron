use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton, rect::Point, EventPump};
use vector2d::Vector2D;

use crate::consts::debug_flags::DEBUG_CAN_CONTROL_CPU;

use super::{
    camera::Camera,
    ent::Owner,
    order::{EntTarget, Order, OrderType},
    selection::MouseCommand,
    world::World,
    world_info::WorldInfo,
};

// TODO: CLEANUP THIS FILE

pub struct Input;

impl Input {
    pub fn process_input(
        event_queue: &mut EventPump,
        camera: &mut Camera,
        world: &mut World,
        world_info: &mut WorldInfo,
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
                    Self::process_mouse_button_down(mouse_btn, x, y, camera, world, world_info);
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
                } => world.selection.clear(&mut world.units),

                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => world.selection.engange_command(MouseCommand::Attack),

                Event::KeyUp {
                    keycode: Some(Keycode::LShift),
                    ..
                } => world.selection.shift_release(),

                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    for unit in &mut world.units {
                        if unit.ent.selected()
                            && (unit.ent.owner == Owner::Player || DEBUG_CAN_CONTROL_CPU)
                        {
                            // Issue stop order to owned selected units
                            unit.stop();
                        }
                    }
                }

                Event::KeyDown {
                    keycode: Some(Keycode::H),
                    ..
                } => {
                    for unit in &mut world.units {
                        if unit.ent.selected()
                            && (unit.ent.owner == Owner::Player || DEBUG_CAN_CONTROL_CPU)
                        {
                            // Issue hold position order to owned selected units
                            unit.hold_position();
                        }
                    }
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
                MouseCommand::Select => world.selection.close(scaled_mouse_pos, &mut world.units),
                MouseCommand::Attack => world.selection.release_command(),
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
        world_info: &mut WorldInfo,
    ) {
        // First, update mouse position
        camera.update_mouse_rect(Point::new(x, y));
        // Then, get scaled mouse position
        let scaled_mouse_pos = camera.get_scaled_mouse_pos();
        // This right click might issue an attack order, we will need to store its possible target
        let mut attack_target = EntTarget {
            ent_id: None,
            ent_rect: None,
            ent_owner: None,
        };
        // Flag to know if we found at least one target
        let mut found_target = false;

        // Check wether we clicked on something attackable
        for unit in &world.units {
            if unit
                .ent
                .get_rect()
                .has_intersection(camera.get_scaled_mouse_rect())
            {
                attack_target = EntTarget {
                    ent_id: Some(unit.ent.id),
                    ent_rect: Some(unit.ent.get_rect()),
                    ent_owner: Some(unit.ent.owner),
                };
                found_target = true;
                break;
            }
        }

        match mouse_btn {
            MouseButton::Left => {
                // Left click
                match world.selection.left_click_command {
                    MouseCommand::Select => {
                        // No command engaged, just open selection
                        world.selection.open(scaled_mouse_pos, &mut world.units);
                    }
                    MouseCommand::Attack => {
                        for unit in &mut world.units {
                            if unit.ent.selected()
                                && (unit.ent.owner == Owner::Player || DEBUG_CAN_CONTROL_CPU)
                            {
                                if !found_target {
                                    // No attack target found; Issue attack move order
                                    let attack_move_order = Order::new(
                                        OrderType::AttackMove,
                                        Vector2D::<f32>::new(
                                            scaled_mouse_pos.x as f32,
                                            scaled_mouse_pos.y as f32,
                                        ),
                                        EntTarget {
                                            ent_id: None,
                                            ent_rect: None,
                                            ent_owner: None,
                                        },
                                    );
                                    unit.add_order(attack_move_order, !world.selection.queueing);
                                } else {
                                    // Attack target found; check if it is a valid one
                                    // (defaults to self in case it's not there, canceling the attack (it should be there tho))
                                    let attack_target_id =
                                        attack_target.ent_id.unwrap_or(unit.ent.id);
                                    if attack_target_id == unit.ent.id {
                                        // Cannot attack yourself!
                                        continue;
                                    }
                                    if attack_target
                                        .ent_owner
                                        .expect(">> Could not get entity team from attack target")
                                        == unit.ent.owner
                                    {
                                        // Cannot attack an ent on the same team!
                                        continue;
                                    }
                                    let attack_order = Order::new(
                                        OrderType::Attack,
                                        Vector2D::<f32>::new(
                                            scaled_mouse_pos.x as f32,
                                            scaled_mouse_pos.y as f32,
                                        ),
                                        EntTarget {
                                            ent_id: Some(attack_target_id),
                                            ent_rect: world_info
                                                .get_ent_rect_by_id(attack_target_id),
                                            ent_owner: world_info
                                                .get_ent_owner_by_id(attack_target_id),
                                        },
                                    );
                                    unit.add_order(attack_order, !world.selection.queueing);
                                }
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
                // Check wether we clicked on something attackable
                for unit in &world.units {
                    if unit
                        .ent
                        .get_rect()
                        .has_intersection(camera.get_scaled_mouse_rect())
                    {
                        attack_target = EntTarget {
                            ent_id: Some(unit.ent.id),
                            ent_rect: Some(unit.ent.get_rect()),
                            ent_owner: Some(unit.ent.owner),
                        };
                        break;
                    }
                }

                for unit in &mut world.units {
                    if unit.ent.selected()
                        && (unit.ent.owner == Owner::Player || DEBUG_CAN_CONTROL_CPU)
                    {
                        if !found_target {
                            // No attack target found; Issue move order
                            let move_order = Order::new(
                                OrderType::Move,
                                Vector2D::<f32>::new(
                                    scaled_mouse_pos.x as f32,
                                    scaled_mouse_pos.y as f32,
                                ),
                                EntTarget {
                                    ent_id: None,
                                    ent_rect: None,
                                    ent_owner: None,
                                },
                            );
                            unit.add_order(move_order, !world.selection.queueing);
                        } else {
                            // Attack target found; check if it is a valid one
                            // (defaults to self in case it's not there, canceling the attack (it should be there tho))
                            let attack_target_id = attack_target.ent_id.unwrap_or(unit.ent.id);
                            if attack_target_id == unit.ent.id {
                                // Cannot attack yourself!
                                continue;
                            }
                            // At this point, we know we will either attack or follow this target, depending on it's team
                            let new_order_type = if attack_target
                                .ent_owner
                                .expect(">> Could not get identity team from attack target")
                                == unit.ent.owner
                            {
                                OrderType::Follow
                            } else {
                                OrderType::Attack
                            };

                            let new_order = Order::new(
                                new_order_type,
                                Vector2D::<f32>::new(
                                    scaled_mouse_pos.x as f32,
                                    scaled_mouse_pos.y as f32,
                                ),
                                EntTarget {
                                    ent_id: Option::Some(attack_target_id),
                                    ent_rect: world_info.get_ent_rect_by_id(attack_target_id),
                                    ent_owner: world_info.get_ent_owner_by_id(attack_target_id),
                                },
                            );
                            unit.add_order(new_order, !world.selection.queueing);
                        }
                    }
                }
            }
            MouseButton::Unknown | MouseButton::X1 | MouseButton::X2 => (),
        }
    }

    fn process_mouse_motion(x: i32, y: i32, camera: &mut Camera, world: &mut World) {
        camera.update_mouse_rect(Point::new(x, y));
        let scaled_mouse_pos = camera.get_scaled_mouse_pos();
        world.selection.tick(scaled_mouse_pos, &mut world.units);
        if camera.is_anchored() {
            let anchored_mouse_pos = camera.get_anchored_mouse_pos();
            camera.drag_to(anchored_mouse_pos);
        }
    }
}
