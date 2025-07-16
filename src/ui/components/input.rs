use egui::{Response, TextEdit, Ui};

/// Custom input field styles
pub struct StyledInput;

impl StyledInput {
    pub fn text(value: &mut String) -> TextEdit {
        TextEdit::singleline(value)
    }

    pub fn password(value: &mut String) -> TextEdit {
        TextEdit::singleline(value).password(true)
    }

    pub fn multiline(value: &mut String) -> TextEdit {
        TextEdit::multiline(value)
    }

    pub fn number(value: &mut String) -> TextEdit {
        TextEdit::singleline(value)
    }
}

/// Input field extensions
pub trait InputExt {
    fn with_placeholder(self, placeholder: &str) -> Self;
    fn with_width(self, width: f32) -> Self;
}

impl<'a> InputExt for TextEdit<'a> {
    fn with_placeholder(self, placeholder: &str) -> Self {
        self.hint_text(placeholder)
    }

    fn with_width(self, width: f32) -> Self {
        self.desired_width(width)
    }
}
