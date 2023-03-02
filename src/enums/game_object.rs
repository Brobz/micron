use crate::structs::{ent::Ent, structure::Structure, unit::Unit};

pub enum GameObject {
    Unit(Ent, Unit),
    Structure(Ent, Structure),
}
