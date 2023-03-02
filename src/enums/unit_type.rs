use crate::structs::unit::Unit;

pub enum UnitType {
    Scout(Unit),
    Worker(Unit),
}

// TODO: Maybe move some of the impl of Unit into Unit type
