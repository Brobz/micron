use super::{selection::Selection, unit::Unit};

pub struct World {
    pub units: Vec<Unit>,
    pub selection: Selection,
}

impl World {
    pub fn new() -> World {
        World {
            units: Vec::<Unit>::new(),
            selection: Selection::new(),
        }
    }
}
