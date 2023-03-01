use super::{game_object::GameObject, selection::Selection};

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
}
