use std::collections::HashMap;

use sdl2::{
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};
use vector2d::Vector2D;

use crate::consts::values::{
    BLACK_RGB, GREEN_RGB, HEALTH_BAR_HEIGHT, HEALTH_BAR_WIDTH, HEALTH_BAR_Y_FLOAT, RED_RGB,
};

use super::ent::{Ent, EntID, EntParentType, Owner};

pub struct WorldInfo {
    ent_max_hp: HashMap<EntID, u32>, // Stores entity max hp,
    ent_hp: HashMap<EntID, f32>,     // Stores entity hp
    pub ent_rect_center: HashMap<EntID, Vector2D<f32>>, // Stores entity rect center
    ent_team: HashMap<EntID, Owner>, // Stores entity team
    ent_parent_type: HashMap<EntID, EntParentType>, // Stores entity parent type
    pub ent_rect: HashMap<EntID, Rect>, // Stores entity rect
}

impl WorldInfo {
    pub fn new() -> Self {
        Self {
            ent_max_hp: HashMap::new(),
            ent_hp: HashMap::new(),
            ent_rect_center: HashMap::new(),
            ent_rect: HashMap::new(),
            ent_parent_type: HashMap::new(),
            ent_team: HashMap::new(),
        }
    }

    pub fn _get_ent_poisition(&self, ent: &Ent) -> Option<Vector2D<f32>> {
        return self.ent_rect_center.get(&ent.id).copied();
    }

    pub fn get_ent_hp(&self, ent: &Ent) -> Option<f32> {
        return self.ent_hp.get(&ent.id).copied();
    }

    pub fn get_ent_rect_center_poisition_by_id(&self, ent_id: EntID) -> Option<Vector2D<f32>> {
        return self.ent_rect_center.get(&ent_id).copied();
    }

    pub fn get_ent_rect_by_id(&self, ent_id: EntID) -> Option<Rect> {
        return self.ent_rect.get(&ent_id).copied();
    }

    pub fn update_ent(&mut self, ent: &Ent) {
        self.clear_ent_by_id(ent.id);
        let ent_rect = ent.get_rect();
        let ent_rect_center = ent_rect.center();
        self.ent_hp.insert(ent.id, ent.hp);
        self.ent_rect_center.insert(
            ent.id,
            Vector2D::new(ent_rect_center.x as f32, ent_rect_center.y as f32),
        );
        self.ent_rect.insert(ent.id, ent_rect);
    }

    pub fn damage_ent(&mut self, ent_id: EntID, dmg: f32) -> Option<f32> {
        self.ent_hp.get_mut(&ent_id)?;

        if let Some(hp) = self.ent_hp.get_mut(&ent_id) {
            let mut new_hp = *hp - dmg;
            if new_hp < 0.0 {
                new_hp = 0.0;
                self.remove_ent_by_id(ent_id);
            } else {
                self.ent_hp.remove(&ent_id);
                self.ent_hp.insert(ent_id, new_hp);
            }
            return Some(new_hp);
        }
        None
    }

    pub fn add_ent(&mut self, ent: &Ent) {
        let ent_rect = ent.get_rect();
        let ent_rect_center = ent_rect.center();
        self.ent_max_hp.insert(ent.id, ent.max_hp);
        self.ent_hp.insert(ent.id, ent.hp);
        self.ent_rect_center.insert(
            ent.id,
            Vector2D::new(ent_rect_center.x as f32, ent_rect_center.y as f32),
        );
        self.ent_rect.insert(ent.id, ent_rect);
        self.ent_team.insert(ent.id, ent.owner);
        self.ent_parent_type.insert(ent.id, ent.parent_type());
    }

    pub fn clear_ent_by_id(&mut self, ent_id: EntID) {
        self.ent_hp.remove(&ent_id);
        self.ent_rect_center.remove(&ent_id);
        self.ent_rect.remove(&ent_id);
    }

    pub fn remove_ent_by_id(&mut self, ent_id: EntID) {
        self.clear_ent_by_id(ent_id);
        self.ent_max_hp.remove(&ent_id);
        self.ent_team.remove(&ent_id);
        self.ent_parent_type.remove(&ent_id);
    }

    pub fn has_ent(&self, ent: &Ent) -> bool {
        self.has_ent_by_id(ent.id)
    }

    pub fn has_ent_by_id(&self, ent_id: EntID) -> bool {
        if self.ent_hp.contains_key(&ent_id) {
            return true;
        }
        false
    }

    pub fn get_ent_owner_by_id(&self, ent_id: EntID) -> Option<Owner> {
        self.ent_team.get(&ent_id).copied()
    }

    pub fn get_ent_parent_type_by_id(&self, ent_id: EntID) -> Option<EntParentType> {
        self.ent_parent_type.get(&ent_id).copied()
    }

    pub fn _get_ent_rect_center_by_id(&self, ent_id: EntID) -> Option<Vector2D<f32>> {
        self.ent_rect_center.get(&ent_id).copied()
    }

    pub fn draw_health_bars(&self, canvas: &mut Canvas<Window>) {
        for ent_id in self.ent_hp.keys() {
            if let Some(health) = self.ent_hp.get(ent_id) {
                if let Some(max_health) = self.ent_max_hp.get(ent_id) {
                    if let Some(pos) = self.ent_rect_center.get(ent_id) {
                        if let Some(parent_type) = self.ent_parent_type.get(ent_id) {
                            // No health bars for ore
                            if *parent_type == EntParentType::Ore {
                                continue;
                            }
                        }
                        let empty_health_bar_rec = Rect::from_center(
                            Point::new(pos.x as i32, (pos.y - HEALTH_BAR_Y_FLOAT) as i32),
                            HEALTH_BAR_WIDTH as u32,
                            HEALTH_BAR_HEIGHT as u32,
                        );
                        let full_health_bar_rec = Rect::from_center(
                            Point::new(
                                (pos.x
                                    - ((1.0 - (health / (*max_health as f32))) * HEALTH_BAR_WIDTH
                                        / 2.0)) as i32,
                                (pos.y - HEALTH_BAR_Y_FLOAT) as i32,
                            ),
                            ((health / (*max_health as f32)) * HEALTH_BAR_WIDTH) as u32,
                            HEALTH_BAR_HEIGHT as u32,
                        );

                        canvas.set_draw_color(RED_RGB);
                        canvas.fill_rect(empty_health_bar_rec).ok();
                        canvas.set_draw_color(GREEN_RGB);
                        canvas.fill_rect(full_health_bar_rec).ok();
                        canvas.set_draw_color(BLACK_RGB);
                        canvas.draw_rect(empty_health_bar_rec).ok();
                    }
                }
            }
        }
    }
}
