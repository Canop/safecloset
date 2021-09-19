use {
    super::*,
    crate::core::OpenDrawer,
    termimad::InputField,
};

/// State of the TUI application relative to drawers
pub enum DrawerState {
    NoneOpen, // no drawer is open
    DrawerCreation(PasswordInputState),
    DrawerOpening(PasswordInputState),
    DrawerEdit(DrawerEditState),
}


impl Default for DrawerState {
    fn default() -> Self {
        Self::NoneOpen
    }
}

impl DrawerState {

    pub fn edit(drawer: OpenDrawer) -> Self {
        Self::DrawerEdit(DrawerEditState::from(drawer))
    }
    #[allow(dead_code)]
    pub fn is_edit(&self) -> bool {
        matches!(self, DrawerState::DrawerEdit(_))
    }
    pub fn input(&mut self) -> Option<&mut InputField> {
        match self {
            Self::DrawerCreation(PasswordInputState { input }) => Some(input),
            Self::DrawerOpening(PasswordInputState { input }) => Some(input),
            Self::DrawerEdit(des) => match &mut des.focus {
                DrawerFocus::NameEdit { input, .. } => Some(input),
                DrawerFocus::ValueEdit { input, .. } => Some(input),
                DrawerFocus::SearchEdit => Some(&mut des.search.input),
                _ => None,
            },
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn has_input(&self) -> bool {
        match self {
            Self::DrawerCreation(PasswordInputState { .. }) => true,
            Self::DrawerOpening(PasswordInputState { .. }) => true,
            Self::DrawerEdit(des) => match &des.focus {
                DrawerFocus::NameEdit { .. } => true,
                DrawerFocus::ValueEdit { .. } => true,
                DrawerFocus::SearchEdit => true,
                _ => false,
            },
            _ => false,
        }
    }
}

