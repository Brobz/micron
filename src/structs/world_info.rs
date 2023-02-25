use std::collections::HashMap;

use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};
use vector2d::Vector2D;

use crate::consts::setup::{HEALTH_BAR_HEIGHT, HEALTH_BAR_WIDTH, HEALTH_BAR_Y_FLOAT};

use super::ent::{Ent, EntID};

pub struct WorldInfo {
    ent_max_hp: HashMap<EntID, u32>,             // Stores entity max hp,
    ent_hp: HashMap<EntID, f32>,                 // Stores entity hp
    ent_position: HashMap<EntID, Vector2D<f32>>, // Stores entity rect center    // Stores entity rect dimensions
}

impl WorldInfo {
    pub fn new() -> WorldInfo {
        WorldInfo {
            ent_max_hp: HashMap::new(),
            ent_hp: HashMap::new(),
            ent_position: HashMap::new(),
        }
    }

    pub fn _get_ent_poisition(&self, ent: &Ent) -> Option<Vector2D<f32>> {
        return self.ent_position.get(&ent.id).copied();
    }

    pub fn get_ent_hp(&self, ent: &Ent) -> Option<f32> {
        return self.ent_hp.get(&ent.id).copied();
    }

    pub fn get_ent_poisition_by_id(&self, ent_id: &EntID) -> Option<Vector2D<f32>> {
        return self.ent_position.get(ent_id).copied();
    }

    pub fn update_ent(&mut self, ent: &Ent) {
        self.clear_ent_by_id(&ent.id);

        let ent_rect = ent.get_rect();
        let ent_rect_center = &ent_rect.center();
        self.ent_hp.insert(ent.id, ent.hp);
        self.ent_position.insert(
            ent.id,
            Vector2D::new(ent_rect_center.x as f32, ent_rect_center.y as f32),
        );
    }

    pub fn damage_ent(&mut self, ent_id: &EntID, dmg: f32) {
        if self.ent_hp.get_mut(ent_id).is_none() {
            return;
        }

        let new_hp = *self.ent_hp.get_mut(ent_id).unwrap() - dmg;
        if new_hp < 0.0 {
            self.remove_ent_by_id(ent_id);
        } else {
            self.ent_hp.remove(ent_id);
            self.ent_hp.insert(*ent_id, new_hp);
        }
    }

    pub fn add_ent(&mut self, ent: &Ent) {
        let ent_rect = ent.get_rect();
        let ent_rect_center = &ent_rect.center();
        self.ent_max_hp.insert(ent.id, ent.max_hp);
        self.ent_hp.insert(ent.id, ent.hp);
        self.ent_position.insert(
            ent.id,
            Vector2D::new(ent_rect_center.x as f32, ent_rect_center.y as f32),
        );
    }

    pub fn clear_ent_by_id(&mut self, ent_id: &EntID) {
        if self.ent_hp.get_mut(ent_id).is_some() {
            self.ent_hp.remove(ent_id);
        }
        if self.ent_position.get_mut(ent_id).is_some() {
            self.ent_position.remove(ent_id);
        }
    }

    pub fn remove_ent_by_id(&mut self, ent_id: &EntID) {
        self.clear_ent_by_id(ent_id);
        if self.ent_max_hp.get_mut(ent_id).is_some() {
            self.ent_max_hp.remove(ent_id);
        }
    }

    pub fn has_ent(&self, ent: &Ent) -> bool {
        self.has_ent_by_id(&ent.id)
    }

    pub fn has_ent_by_id(&self, ent_id: &EntID) -> bool {
        if self.ent_hp.get(ent_id).is_some() {
            return true;
        }
        false
    }

    pub fn draw_health_bars(&self, canvas: &mut Canvas<Window>) {
        for ent_id in self.ent_hp.keys() {
            let health = self.ent_hp.get(ent_id).unwrap();
            let max_health = self.ent_max_hp.get(ent_id).unwrap();
            let pos = self.ent_position.get(ent_id).unwrap();
            let empty_health_bar_rec = Rect::from_center(
                Point::new(pos.x as i32, (pos.y - HEALTH_BAR_Y_FLOAT) as i32),
                HEALTH_BAR_WIDTH as u32,
                HEALTH_BAR_HEIGHT as u32,
            );
            let full_health_bar_rec = Rect::from_center(
                Point::new(
                    (pos.x - ((1.0 - (health / (*max_health as f32))) * HEALTH_BAR_WIDTH / 2.0))
                        as i32,
                    (pos.y - HEALTH_BAR_Y_FLOAT) as i32,
                ),
                ((health / (*max_health as f32)) * HEALTH_BAR_WIDTH) as u32,
                HEALTH_BAR_HEIGHT as u32,
            );

            canvas.set_draw_color(Color::RED);
            canvas.fill_rect(empty_health_bar_rec).ok();
            canvas.set_draw_color(Color::GREEN);
            canvas.fill_rect(full_health_bar_rec).ok();
            canvas.set_draw_color(Color::BLACK);
            canvas.draw_rect(empty_health_bar_rec).ok();
        }
    }
}
