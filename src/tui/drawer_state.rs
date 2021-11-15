use {
    super::*,
    crate::core::OpenDrawer,
    termimad::InputField,
};

/// State of the TUI application relative to drawers
// FIXME just use an option<drawereditstate> dans app_state ?
#[allow(clippy::large_enum_variant)]
pub enum DrawerState {
    NoneOpen, // no drawer is open
    // DrawerCreation(PasswordInputState),
    // DrawerOpening(PasswordInputState),
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
    pub fn is_edit(&self) -> bool {
        matches!(self, DrawerState::DrawerEdit(_))
    }
    pub fn is_pending_removal(&self) -> bool {
        matches!(
            self,
            Self::DrawerEdit(DrawerEditState { focus: DrawerFocus::PendingRemoval{..}, .. }),
        )
    }
    pub fn input(&mut self) -> Option<&mut InputField> {
        match self {
            Self::DrawerEdit(des) => match &mut des.focus {
                DrawerFocus::NameEdit { input, .. } => Some(input),
                DrawerFocus::ValueEdit { input, .. } => Some(input),
                DrawerFocus::SearchEdit { .. } => Some(&mut des.search.input),
                _ => None,
            },
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn is_on_entry_value(&mut self) -> bool {
        match self {
            Self::DrawerEdit(des) => match &mut des.focus {
                DrawerFocus::NameEdit { .. } => true,
                DrawerFocus::ValueEdit { .. } => true,
                _ => false,
            },
            _ => false,
        }
    }
    #[allow(dead_code)]
    pub fn has_input(&self) -> bool {
        match self {
            Self::DrawerEdit(des) => match &des.focus {
                DrawerFocus::NameEdit { .. } => true,
                DrawerFocus::ValueEdit { .. } => true,
                DrawerFocus::SearchEdit { .. } => true,
                _ => false,
            },
            _ => false,
        }
    }
}

