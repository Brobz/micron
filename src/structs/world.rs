use sdl2::{rect::Rect, render::Canvas, video::Window};

use crate::{
    consts::values::{MAP_HEIGHT, MAP_PADDING, MAP_WIDTH, SCREEN_BACKGROUND_COLOR},
    enums::{game_object::GameObject, unit_type::UnitType},
};

use super::{camera::Camera, ent::EntID, selection::Selection, world_info::WorldInfo};

pub struct World {
    pub game_objects: Vec<GameObject>,
    pub selection: Selection,
}

impl World {
    pub fn new() -> Self {
        Self {
            game_objects: Vec::<GameObject>::new(),
            selection: Selection::new(),
        }
    }

    // TODO: Breakup this method into smaller methods
    pub fn tick(&mut self, world_info: &mut WorldInfo) {
        // Tick units
        // Store a list of any new gameobjects that are to be spawned after this tick
        let mut game_object_spawn_list: Vec<GameObject> = Vec::<GameObject>::new();
        // Also, store the index of any units that are to be removed after this tick
        let mut ent_cleanup_list: Vec<EntID> = Vec::<EntID>::new();
        for game_object in &mut self.game_objects {
            match game_object {
                GameObject::Unit(ent, unit) => {
                    // Check if this unit's entity still exists in the world
                    if world_info.has_ent(ent) {
                        match unit {
                            UnitType::Scout(unit)
                            | UnitType::Miner(unit)
                            | UnitType::Collector(unit) => {
                                // If so, tick and update world_info
                                unit.tick(ent, world_info);
                                world_info.update_ent(ent);
                            }
                        }
                    } else {
                        // If not, add to cleanup list
                        ent_cleanup_list.push(ent.id);
                    }
                }
                GameObject::OrePatch(ent, ore_patch) =>
                // Check if this ore patch's entity still exists in the world
                {
                    if world_info.has_ent(ent) {
                        // If so, tick and update world_info
                        if let Some(new_ore) = ore_patch.tick(ent, world_info) {
                            game_object_spawn_list.push(new_ore);
                        }
                        world_info.update_ent(ent);
                    } else {
                        // If not, add to cleanup list
                        ent_cleanup_list.push(ent.id);
                    }
                }
                GameObject::Ore(ent, ore) =>
                // Check if this ore's entity still exists in the world
                {
                    if world_info.has_ent(ent) {
                        // If so, tick and update world_info
                        ore.tick(ent, world_info);
                        world_info.update_ent(ent);
                    } else {
                        // If not, add to cleanup list
                        ent_cleanup_list.push(ent.id);
                    }
                }
                GameObject::Structure(_ent, _structure) => (),
            }
        }

        // Spawn new game objects
        self.game_objects.append(&mut game_object_spawn_list);

        // Remove dead game objects
        // TODO: collectors must drop all storage on death on an ore ball
        self.game_objects.retain(|game_object| match game_object {
            GameObject::Unit(ent, _)
            | GameObject::Structure(ent, _)
            | GameObject::OrePatch(ent, _)
            | GameObject::Ore(ent, _) => !ent_cleanup_list.contains(&ent.id),
        });

        // Tick orders
        for game_object in &mut self.game_objects {
            match game_object {
                GameObject::Unit(ent, _) | GameObject::Structure(ent, _) => {
                    for order in &mut ent.orders {
                        // If this order has no ent target, skip it
                        if order.ent_target.ent_id.is_none() {
                            continue;
                        }

                        // Grab the target EntID
                        if let Some(ent_target_id) = order.ent_target.ent_id {
                            // Update the order's target position to the attacked entity's position (if available)
                            if let Some(target_position) =
                                world_info.get_ent_rect_center_poisition_by_id(ent_target_id)
                            {
                                order.current_move_target = target_position;
                            }

                            // Also update the attack target rect
                            order.ent_target.ent_rect =
                                world_info.get_ent_rect_by_id(ent_target_id);

                            // Note: no need to update target's team! for now...
                        }
                    }
                }
                _ => (),
            }
        }
    }

    pub fn draw(
        &mut self,
        canvas: &mut Canvas<Window>,
        world_info: &mut WorldInfo,
        camera: &mut Camera,
    ) {
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
        for game_object in &mut self.game_objects {
            match game_object {
                GameObject::Unit(ent, unit) => match unit {
                    UnitType::Scout(unit) | UnitType::Miner(unit) | UnitType::Collector(unit) => {
                        unit.draw_orders(ent, canvas)
                    }
                },
                GameObject::Structure(_ent, _structure) => todo!(),
                GameObject::OrePatch(_ent, _ore) => (),
                GameObject::Ore(_ent, _ore) => (),
            }
        }

        // Draw game_objects
        for game_object in &mut self.game_objects {
            match game_object {
                GameObject::Unit(ent, unit) => match unit {
                    UnitType::Scout(unit) | UnitType::Miner(unit) | UnitType::Collector(unit) => {
                        unit.draw(ent, canvas)
                    }
                },
                GameObject::OrePatch(ent, ore_patch) => ore_patch.draw(ent, canvas),
                GameObject::Ore(ent, ore) => ore.draw(ent, canvas),
                GameObject::Structure(_ent, _structure) => todo!(),
            }
        }

        // Draw attack lines
        for game_object in &mut self.game_objects {
            match game_object {
                GameObject::Unit(ent, unit) => match unit {
                    UnitType::Scout(unit) | UnitType::Miner(unit) | UnitType::Collector(unit) => {
                        unit.draw_attack_lines(ent, canvas)
                    }
                },
                GameObject::Structure(_ent, _structure) => todo!(),
                GameObject::OrePatch(_ent, _ore) => (),
                GameObject::Ore(_ent, _) => (),
            }
        }

        // Draw Health Bars
        world_info.draw_health_bars(canvas);

        // Draw selection box
        self.selection.draw(canvas);
    }
}
