use egui::{Button, Response, Ui};

/// Custom button styles for the application
pub struct StyledButton;

impl StyledButton {
    pub fn primary(text: &str) -> Button {
        Button::new(text)
    }

    pub fn secondary(text: &str) -> Button {
        Button::new(text)
    }

    pub fn danger(text: &str) -> Button {
        Button::new(text)
    }

    pub fn success(text: &str) -> Button {
        Button::new(text)
    }
}

/// Button extensions
pub trait ButtonExt {
    fn primary(self) -> Self;
    fn secondary(self) -> Self;
    fn danger(self) -> Self;
    fn success(self) -> Self;
}

impl ButtonExt for Button<'_> {
    fn primary(self) -> Self {
        self
    }

    fn secondary(self) -> Self {
        self
    }

    fn danger(self) -> Self {
        self
    }

    fn success(self) -> Self {
        self
    }
}
