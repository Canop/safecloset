mod password_dialog_purpose;
mod password_dialog_state;
mod password_dialog_view;

pub use {
    password_dialog_purpose::*,
    password_dialog_state::*,
    password_dialog_view::*,
};

use {
    super::*,
    crossterm::event::KeyEvent,
};

pub struct PasswordDialog {
    state: PasswordDialogState,
    pub view: PasswordDialogView,
}

impl PasswordDialog {
    pub fn new(
        purpose: PasswordDialogPurpose,
        hide_chars: bool,
    ) -> Self {
        let state = PasswordDialogState::new(purpose, hide_chars);
        let view = PasswordDialogView::default();
        Self { state, view }
    }
    pub fn toggle_hide_chars(&mut self) {
        self.state.password.password_mode ^= true;
    }
    pub fn get_password(&self) -> String {
        self.state.get_password()
    }
    pub fn purpose(&self) -> PasswordDialogPurpose {
        self.state.purpose
    }
    pub fn apply_key_event(&mut self, key: KeyEvent) -> bool {
        self.state.apply_key_event(key)
    }
    pub fn draw(
        &mut self,
        w: &mut W,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        self.view.draw(w, &mut self.state, app_skin)
    }
}


