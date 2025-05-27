use {
    super::*,
    crate::tui::ContentSkin,
    crokey::{
        KeyCombination,
        crossterm::event::MouseEvent,
    },
    termimad::*,
};

pub struct PasswordDialogState {
    pub purpose: PasswordDialogPurpose,
    pub password: InputField,
}

impl PasswordDialogState {
    pub fn new(
        purpose: PasswordDialogPurpose,
        hide_chars: bool,
    ) -> Self {
        let mut password = ContentSkin::make_input();
        password.password_mode = hide_chars;
        Self { purpose, password }
    }
    pub fn get_password(&self) -> String {
        self.password.get_content()
    }
    pub fn apply_key_event(
        &mut self,
        key: KeyCombination,
    ) -> bool {
        self.password.apply_key_combination(key)
    }
    /// handle a mouse event
    pub fn on_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        double_click: bool,
    ) {
        self.password.apply_mouse_event(mouse_event, double_click);
    }
}
