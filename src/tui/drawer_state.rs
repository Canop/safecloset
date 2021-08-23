use {
    super::*,
    crate::{core::*, error::SafeClosetError},
    termimad::{Area, InputField},
};

/// State of the TUI application relative to drawers
pub enum DrawerState {
    NoneOpen, // no drawer is open
    DrawerCreation(PasswordInputState),
    DrawerOpening(PasswordInputState),
    DrawerEdit(DrawerEditState),
    // EntrySearch
}

/// State of the application when a drawer is open.
///
/// Contains the open drawer and an entry state.
pub struct DrawerEditState {
    pub drawer: OpenDrawer,
    pub scroll: usize, // number of lines hidden above the top of the view
    pub page_height: Option<usize>, // number of lines which can be seen
    pub entry_state: EntryState,
    edit_count: usize, // a counter to know whether the drawer changed
}

impl Default for DrawerState {
    fn default() -> Self {
        Self::NoneOpen
    }
}

impl From<OpenDrawer> for DrawerEditState {
    fn from(drawer: OpenDrawer) -> Self {
        Self {
            drawer,
            scroll: 0,
            page_height: None,
            entry_state: EntryState::NoneSelected,
            edit_count: 0,
        }
    }
}

impl DrawerState {
    pub fn input(&mut self) -> Option<&mut InputField> {
        match self {
            Self::DrawerCreation(PasswordInputState { input }) => Some(input),
            Self::DrawerOpening(PasswordInputState { input }) => Some(input),
            Self::DrawerEdit(des) => match &mut des.entry_state {
                EntryState::NameEdit { input, .. } => Some(input),
                EntryState::ValueEdit { input, .. } => Some(input),
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
            Self::DrawerEdit(des) => match &des.entry_state {
                EntryState::NameEdit { .. } => true,
                EntryState::ValueEdit { .. } => true,
                _ => false,
            },
            _ => false,
        }
    }
}

impl DrawerEditState {
    /// Must be called on starting editing a name or value
    pub fn increment_edit_count(&mut self) {
        self.edit_count += 1;
    }
    /// Must be called on cancelling a name or value edition
    pub fn decrement_edit_count(&mut self) {
        if self.edit_count > 0 {
            self.edit_count -= 1;
        } else {
            warn!("internal error: edit count decremented when nul");
        }
    }
    /// Tells whether the content was edited since opening
    /// (it may be equal)
    pub fn touched(&self) -> bool {
        self.edit_count > 0
    }
    /// Ensure the scroll is consistent with the size of content
    /// and terminal height, and that the selection is visible, if any.
    ///
    /// It's not necessary to call this other than from set_page_height
    /// as this function is called before all drawings.
    fn fix_scroll(&mut self) {
        if let Some(page_height) = self.page_height {
            self.scroll = fix_scroll(
                self.scroll,
                self.entry_state.idx(),
                self.drawer.entries.len(),
                page_height,
            );
        }
    }
    pub fn set_page_height(&mut self, page_height: usize) {
        self.page_height = Some(page_height);
        self.fix_scroll();
    }
    /// Save the drawer, which closes it, then reopen it, keeping the
    /// same state around (scroll and selection)
    pub fn close_and_reopen(self, closet: &mut Closet) -> Result<Self, SafeClosetError> {
        let DrawerEditState {
            drawer,
            scroll,
            page_height,
            entry_state,
            ..
        } = self;
        let drawer = closet.close_then_reopen(drawer)?;
        Ok(DrawerEditState {
            drawer,
            scroll,
            page_height,
            entry_state,
            edit_count: 0,
        })
    }
    pub fn edit_entry_name(&mut self, idx: usize) {
        let mut input = InputField::new(Area::uninitialized());
        input.set_content(&self.drawer.entries[idx].name);
        self.entry_state = EntryState::NameEdit { idx, input };
        self.increment_edit_count();
    }
    pub fn edit_entry_value(&mut self, idx: usize) {
        let mut input = InputField::new(Area::uninitialized());
        input.set_content(&self.drawer.entries[idx].value);
        self.entry_state = EntryState::ValueEdit { idx, input };
        self.increment_edit_count();
    }
    pub fn apply_scroll_command(&mut self, scroll_command: ScrollCommand) {
        if let Some(page_height) = self.page_height {
            self.scroll = scroll_command.apply(self.scroll, self.drawer.entries.len(), page_height);
        }
    }
    pub fn close_input(&mut self, discard: bool) -> bool {
        if let EntryState::NameEdit { idx, input } = &self.entry_state {
            let idx = *idx;
            if !discard {
                let new_name = input.get_content();
                if new_name == self.drawer.entries[idx].name {
                    self.decrement_edit_count();
                } else {
                    self.drawer.entries[idx].name = new_name;
                }
            }
            self.entry_state = EntryState::NameSelected { idx };
            return true;
        }
        if let EntryState::ValueEdit { idx, input } = &self.entry_state {
            let idx = *idx;
            if !discard {
                let new_value = input.get_content();
                if new_value == self.drawer.entries[idx].value {
                    self.decrement_edit_count();
                } else {
                    self.drawer.entries[idx].value = new_value;
                }
            }
            self.entry_state = EntryState::ValueSelected { idx };
            return true;
        }
        false
    }
}
