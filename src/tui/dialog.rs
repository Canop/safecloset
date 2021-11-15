use super::*;

/// the dialog that may be displayed over the drawer
pub enum Dialog {
    None,
    Menu(Menu),
    Help(Help),
    Password(PasswordDialog),
}

impl Dialog {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
    pub fn is_help(&self) -> bool {
        matches!(self, Self::Help(_))
    }
}
