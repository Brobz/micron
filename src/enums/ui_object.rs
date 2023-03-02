use crate::structs::{text_label::TextLabel, ui_element::UIElement};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Button {}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum UIObject {
    TextLabel(UIElement, TextLabel),
    Button(UIElement, Button),
}
