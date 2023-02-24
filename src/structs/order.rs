use vector2d::Vector2D;

#[derive(Copy, Clone, PartialEq)]
pub enum OrderType {
    Move,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Order {
    pub order_type: OrderType,
    pub executed: bool,
    pub completed: bool,
    pub target: Vector2D<f32>,
}

impl Order {
    pub fn new(order_type: OrderType, target: Vector2D<f32>) -> Order {
        Order {
            order_type,
            executed: false,
            completed: false,
            target,
        }
    }

    pub fn execute(&mut self) {
        self.executed = true;
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }
}
