use sdl2::{pixels::Color, rect::Rect};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum UIElementID {
    DEBUG_EntCount,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct UIElement {
    pub id: UIElementID, // What does this UI element represent?
    pub label: String,   // Text label for the UI element
    pub color: Color,    // General (text) color of the UI element
    pub rect: Rect,      // Position and dimension of the UI element
    pub visible: bool,   // Flag to turn rendering on or off for this particular UIElement
}

impl UIElement {
    pub fn new(id: UIElementID, label: String, color: Color, rect: Rect) -> Self {
        Self {
            id,
            label,
            rect,
            color,
            visible: true,
        }
    }

    pub fn set_label(&mut self, new_label: String) {
        self.label = new_label;
    }
}
