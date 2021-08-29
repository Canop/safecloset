use {
    super::*,
    crate::{
        core::*,
        error::SafeClosetError,
        search::*,
    },
    termimad::{Area, InputField},
};

/// State of the application when a drawer is open.
///
/// Contains the open drawer and an entry state.
pub struct DrawerEditState {
    pub drawer: OpenDrawer,
    pub scroll: usize, // number of lines hidden above the top of the view
    pub page_height: Option<usize>, // number of lines which can be seen
    pub focus: DrawerFocus,
    edit_count: usize, // a counter to know whether the drawer changed
    pub search: SearchState,
}

impl From<OpenDrawer> for DrawerEditState {
    fn from(drawer: OpenDrawer) -> Self {
        Self {
            drawer,
            scroll: 0,
            page_height: None,
            focus: DrawerFocus::NoneSelected,
            edit_count: 0,
            search: SearchState::default(),
        }
    }
}

impl DrawerEditState {

    pub fn listed_entry_idx(&self, line: usize) -> Option<usize> {
        if let Some(search_result) = &self.search.result {
            search_result
                .entries
                .get(line)
                .map(|MatchingEntry { idx, .. }| *idx)
        } else if line < self.drawer.entries.len() {
            Some(line)
        } else {
            None
        }
    }
    pub fn listed_entry(&self, line: usize) -> Option<(usize, Option<NameMatch>)> {
        if let Some(search_result) = &self.search.result {
            search_result
                .entries
                .get(line)
                .map(|MatchingEntry { idx, name_match }| (*idx, Some(name_match.clone())))
        } else if line < self.drawer.entries.len() {
            Some((line, None))
        } else {
            None
        }
    }
    /// return the number of lines which should be displayed in the entries list, taking
    /// the search into account
    pub fn listed_entries_count(&self) -> usize {
        if let Some(search_result) = &self.search.result {
            search_result.entries.len()
        } else {
            self.drawer.entries.len()
        }
    }

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
                self.focus.line(),
                self.listed_entries_count(),
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
            focus,
            search,
            ..
        } = self;
        let drawer = closet.close_then_reopen(drawer)?;
        Ok(DrawerEditState {
            drawer,
            scroll,
            page_height,
            focus,
            search,
            edit_count: 0,
        })
    }
    pub fn edit_entry_name_by_line(&mut self, line: usize) {
        if let Some(idx) = self.listed_entry_idx(line) {
            let mut input = InputField::new(Area::uninitialized());
            input.set_content(&self.drawer.entries[idx].name);
            self.focus = DrawerFocus::NameEdit { line, input };
            self.increment_edit_count();
        }
    }
    pub fn edit_entry_value_by_line(&mut self, line: usize) {
        if let Some(idx) = self.listed_entry_idx(line) {
            let mut input = InputField::new(Area::uninitialized());
            input.set_content(&self.drawer.entries[idx].value);
            self.focus = DrawerFocus::ValueEdit { line, input };
            self.increment_edit_count();
        }
    }
    pub fn apply_scroll_command(&mut self, scroll_command: ScrollCommand) {
        if let Some(page_height) = self.page_height {
            self.scroll = scroll_command.apply(self.scroll, self.listed_entries_count(), page_height);
        }
    }
    pub fn close_input(&mut self, discard: bool) -> bool {
        if let DrawerFocus::NameEdit { line, input } = &self.focus {
            let line = *line;
            if let Some(idx) = self.listed_entry_idx(line) {
                if !discard {
                    let new_name = input.get_content();
                    if new_name == self.drawer.entries[idx].name {
                        self.decrement_edit_count();
                    } else {
                        self.drawer.entries[idx].name = new_name;
                    }
                }
                self.focus = DrawerFocus::NameSelected { line };
                return true;
            }
        }
        if let DrawerFocus::ValueEdit { line, input } = &self.focus {
            let line = *line;
            if let Some(idx) = self.listed_entry_idx(line) {
                if !discard {
                    let new_value = input.get_content();
                    if new_value == self.drawer.entries[idx].value {
                        self.decrement_edit_count();
                    } else {
                        self.drawer.entries[idx].value = new_value;
                    }
                }
                self.focus = DrawerFocus::ValueSelected { line };
                return true;
            }
        }
        if self.focus.is_search() {
            self.focus = match self.best_search_line() {
                Some(line) => DrawerFocus::NameSelected { line },
                None => DrawerFocus::NoneSelected,
            };
            if discard {
                // FIXME be back to previous focus ?
                self.search.clear();
            }
            self.search.update(&self.drawer);
            return true;
        }
        false
    }
    pub fn has_best_search(&self, line: usize) -> bool {
        self.best_search_line()
            .map_or(false, |l| l == line)
    }
    pub fn best_search_line(&self) -> Option<usize> {
        if self.focus.is_search() {
            self.search.result.as_ref().and_then(|r| r.best_line)
        } else {
            None
        }
    }
}

