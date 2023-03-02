use crate::structs::{ent::Ent, ore::Ore, structure::Structure, unit::Unit};

pub enum GameObject {
    Unit(Ent, Unit),
    Structure(Ent, Structure),
    Ore(Ent, Ore),
}
