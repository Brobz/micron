use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::BlendMode;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::consts::helper::find_selection_box_translation;
use crate::consts::values::SELECTION_BOX_COLOR;
use crate::Unit;

pub enum MouseCommand {
    Select,
    Attack,
}

// This resource tracks the current selection of units and structures
pub struct Selection {
    pub open: bool,
    pub just_closed: bool,
    pub queueing: bool,
    pub clearing: bool,
    pub origin: Point,
    pub center: Point,
    pub selection_box: Rect,
    pub left_click_command: MouseCommand,
}

impl Selection {
    pub fn new() -> Self {
        Self {
            open: false,
            just_closed: false,
            clearing: false,
            origin: Point::new(-1, -1),
            center: Point::new(-1, -1),
            selection_box: Rect::new(-1, -1, 0, 0),
            queueing: false,
            left_click_command: MouseCommand::Select,
        }
    }
    pub fn tick(&mut self, mouse_position: Point, units: &mut Vec<Unit>) {
        if self.open {
            let new_pos = find_selection_box_translation(mouse_position, self.origin);
            self.selection_box.set_x(new_pos.x);
            self.selection_box.set_y(new_pos.y);
            self.selection_box
                .set_width((mouse_position.x - self.origin.x).unsigned_abs());
            self.selection_box
                .set_height((mouse_position.y - self.origin.y).unsigned_abs());
            self.center = self.selection_box.center();
        } else if self.just_closed {
            let mut at_least_one_selected = false;
            let mut units_to_deselect: Vec<&mut Unit> = Vec::<&mut Unit>::new();
            for unit in units {
                let possible_intersection = unit.ent.get_rect().intersection(self.selection_box);
                if possible_intersection.is_some() {
                    unit.select();
                    at_least_one_selected = true;
                } else {
                    units_to_deselect.push(unit);
                }
            }
            if at_least_one_selected {
                for unit in units_to_deselect {
                    if !self.queueing {
                        unit.deselect();
                    }
                }
            }
            self.just_closed = false;
        } else if self.clearing {
            for unit in units {
                unit.deselect();
            }
            self.clearing = false;
        } else {
            self.origin = mouse_position;
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        if !self.open {
            return {};
        };
        canvas.set_blend_mode(BlendMode::Blend);
        canvas.set_draw_color(SELECTION_BOX_COLOR);
        canvas.fill_rect(self.selection_box).ok();
        canvas.set_blend_mode(BlendMode::None);
    }

    pub fn open(&mut self, mouse_position: Point, units: &mut Vec<Unit>) {
        self.open = true;
        self.tick(mouse_position, units);
    }

    pub fn close(&mut self, mouse_position: Point, units: &mut Vec<Unit>) {
        self.open = false;
        self.just_closed = true;
        self.tick(mouse_position, units);
    }

    pub fn clear(&mut self) {
        self.clearing = true;
    }

    pub fn shift_press(&mut self) {
        self.queueing = true;
    }

    pub fn shift_release(&mut self) {
        self.queueing = false;
        self.release_command();
    }

    pub fn engange_command(&mut self, command: MouseCommand) {
        if !self.open {
            self.left_click_command = command;
        }
    }

    pub fn release_command(&mut self) {
        if !self.queueing {
            self.left_click_command = MouseCommand::Select;
        }
    }
}
