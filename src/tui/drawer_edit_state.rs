use {
    super::*,
    crate::{
        core::*,
        error::SafeClosetError,
        search::*,
    },
    termimad::{Area, InputField, FmtText},
};

/// State of the application when a drawer is open.
///
/// Contains the open drawer and an entry state.
pub struct DrawerEditState {
    pub drawer: OpenDrawer,
    pub scroll: usize, // number of lines hidden above the top of the view
    pub focus: DrawerFocus,
    edit_count: usize, // a counter to know whether the drawer changed
    pub search: SearchState,
    layout: DrawerDrawingLayout,
}

#[derive(Debug, Clone, Copy)]
pub enum EditionPos {
    Start,
    End,
}
impl EditionPos {
    pub fn apply_to_input(self, input: &mut InputField) {
        match self {
            Self::Start => {
                input.move_to_start();
            }
            Self::End => {
                input.move_to_end();
            }
        }
    }
}

impl DrawerEditState {

    pub fn from(drawer: OpenDrawer) -> Self {
        Self {
            drawer,
            scroll: 0,
            focus: DrawerFocus::NoneSelected,
            edit_count: 0,
            search: SearchState::default(),
            layout: DrawerDrawingLayout::default(),
        }
    }

    pub fn move_line(&mut self, dir: Direction) {
        use DrawerFocus::*;
        match dir {
            Direction::Up => {
                if self.focus.is_search() {
                    if let Some(line) = self.best_search_line() {
                        let line = if line > 0 {
                            line - 1
                        } else {
                            self.listed_entries_count() - 1
                        };
                        self.focus = NameSelected { line };
                    } else {
                        // there's no match, so there's no point to keep the search
                        self.search.clear();
                        self.search.update(&self.drawer);
                        self.focus = NameSelected { line: 0 };
                    }
                    return;
                }
                if let NameSelected { line } = &self.focus {
                    let line = if *line > 0 {
                        line - 1
                    } else {
                        self.listed_entries_count() - 1
                    };
                    self.focus = NameSelected { line };
                }
                if let ValueSelected { line } = &self.focus {
                    let line = if *line > 0 {
                        line - 1
                    } else {
                        self.listed_entries_count() - 1
                    };
                    self.focus = ValueSelected { line };
                }
                if matches!(self.focus, NoneSelected) {
                    self.focus = NameSelected { line: 0 };
                }
            }
            Direction::Down => {
                if self.focus.is_search() {
                    if let Some(line) = self.best_search_line() {
                        let line = if line < self.listed_entries_count() {
                            line + 1
                        } else {
                            0
                        };
                        self.focus = NameSelected { line };
                    } else {
                        // there's no match, so there's no point to keep the search
                        self.search.clear();
                        self.search.update(&self.drawer);
                        self.focus = NameSelected { line: 0 };
                    }
                    return;
                }
                if let NameSelected { line } = &self.focus {
                    let line = if *line + 1 < self.listed_entries_count() {
                        line + 1
                    } else {
                        0
                    };
                    self.focus = NameSelected { line };
                }
                if let ValueSelected { line } = &self.focus {
                    let line = if *line + 1 < self.listed_entries_count() {
                        line + 1
                    } else {
                        0
                    };
                    self.focus = ValueSelected { line };
                }
                if matches!(self.focus, NoneSelected) {
                    self.focus = NameSelected { line: 0 };
                }
            }
        }
    }


    /// update the drawer drawing layout.
    ///
    /// Must be called before every drawing as it depends on about everything:
    /// - the content area in which it will be
    /// - the filtering state
    /// - the settings
    /// - the entries
    pub fn update_drawing_layout(
        &mut self,
        content_area: &Area,
    ) {
        debug_assert!(content_area.height > 3);
        self.layout.lines_area.left = 0;
        self.layout.lines_area.top = content_area.top + 3;
        let page_height = content_area.height - 3;
        self.layout.lines_area.width = content_area.width;
        self.layout.lines_area.height = page_height;
        self.layout.name_width = (self.layout.lines_area.width / 3).min(30);
        let dc = &self.drawer.content;
        let open_all_values = dc.settings.open_all_values && !dc.settings.hide_values;
        let value_width = self.layout.value_width();
        let lines_count = self.listed_entries_count();
        self.layout.content_height = 0;
        self.layout.value_heights_by_line.clear();
        for l in 0..lines_count {
            let idx = self.listed_entry_idx(l).unwrap(); // SAFETY: we iter among valid lines
            let height = match &self.focus {
                DrawerFocus::ValueEdit { input, line } if l == *line => {
                    // this line's value is edited, its height is given by the
                    // number of lines computed by the input
                    input.content().line_count().min(page_height as usize - 5)
                }
                _ => {
                    let open = open_all_values || self.focus.is_value_selected(l);
                    if open {
                        // we compute the number of lines the text would be for
                        // the available width, taking wrapping into account
                        let text = FmtText::from(
                            termimad::get_default_skin(),
                            &self.drawer.content.entries[idx].value,
                            Some(value_width),
                        );
                        text.lines.len().max(1).min(page_height as usize - 5)
                    } else {
                        // this line's value is neither open nor selected, we display
                        // just the first line of the value (or a line of squares if
                        // unselected values are hidden)
                        1
                    }
                }
            };
            self.layout.content_height += height;
            self.layout.value_heights_by_line.push(height);
        }
        self.fix_scroll();
        self.layout.has_scrollbar = self.content_height() > self.page_height();
    }

    pub fn update_search(&mut self) {
        self.search.update(&self.drawer)
    }

    pub fn clicked_line(&self, y: usize) -> Option<usize> {
        let heights = self.layout.value_heights_by_line
            .iter()
            .enumerate()
            .skip(self.scroll);
        let mut sum_heights = self.layout.lines_area.top as usize;
        for (line, height) in heights {
            sum_heights += height;
            if sum_heights > y {
                return Some(line);
            }
        }
        None
    }

    pub fn scrollbar(&self) -> Option<(u16, u16)> {
        self.layout.lines_area.scrollbar(self.scroll, self.content_height())
    }
    fn page_height(&self) -> usize {
        self.layout.lines_area.height as usize
    }
    pub fn layout(&self) -> DrawerDrawingLayout {
        self.layout.clone()
    }

    #[allow(dead_code)]
    pub fn current_cell(&self) -> Option<&str> {
        use DrawerFocus::*;
        match &self.focus {
            NameSelected { line } | NameEdit { line, .. } => {
                self.listed_entry_idx(*line)
                    .and_then(|idx| self.drawer.content.entries.get(idx))
                    .map(|entry| entry.name.as_str())
            }
            ValueSelected { line } | ValueEdit { line, .. } => {
                self.listed_entry_idx(*line)
                    .and_then(|idx| self.drawer.content.entries.get(idx))
                    .map(|entry| entry.value.as_str())
            }
            _ => {
                None
            }
        }
    }

    /// Give the index of the entry from its line among the listed
    /// entries (either all entries or only the ones matching if there's
    /// a search)
    pub fn listed_entry_idx(&self, line: usize) -> Option<usize> {
        if let Some(search_result) = &self.search.result {
            search_result
                .entries
                .get(line)
                .map(|MatchingEntry { idx, .. }| *idx)
        } else if line < self.drawer.content.entries.len() {
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
        } else if line < self.drawer.content.entries.len() {
            Some((line, None))
        } else {
            None
        }
    }
    /// return the number of lines which should be displayed in the entries list, taking
    /// filtering into account
    pub fn listed_entries_count(&self) -> usize {
        if let Some(search_result) = &self.search.result {
            search_result.entries.len()
        } else {
            self.drawer.content.entries.len()
        }
    }
    /// return the total height of listed entries (the ones not filtered out)
    pub fn content_height(&self) -> usize {
        self.layout.content_height
    }
    /// return the total height of listed entries before the given one
    pub fn content_height_before(&self, line: usize) -> usize {
        if line == 0 {
            0
        } else {
            self.layout.value_heights_by_line
                .iter()
                .take(line - 1)
                .sum()
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
    /// Tell whether the content was edited since opening
    /// (it may be equal)
    pub fn touched(&self) -> bool {
        match self.edit_count {
            0 => false,
            1 => {
                // we may have entered an input but done no real change
                match &self.focus {
                    DrawerFocus::NameEdit { line, input } => {
                        self.listed_entry_idx(*line)
                            .map_or(true, |idx| !input.is_content(&self.drawer.content.entries[idx].name))
                    }
                    DrawerFocus::ValueEdit { line, input } => {
                        self.listed_entry_idx(*line)
                            .map_or(true, |idx| !input.is_content(&self.drawer.content.entries[idx].value))
                    }
                    _ => true,
                }
            }
            _ => true,
        }
    }
    /// Ensure the scroll is consistent with the size of content
    /// and terminal height, and that the selection is visible, if any.
    ///
    /// It's not necessary to call this other than from set_page_height
    /// (which is called before all drawings).
    fn fix_scroll(&mut self) {
        let content_height = self.layout.content_height;
        let page_height = self.page_height();
        let heights = &self.layout.value_heights_by_line;
        if content_height <= page_height {
            self.scroll = 0;
        } else if let Some(selection) = self.focus.line() {
            let hbl = self.content_height_before(selection);
            if hbl < self.scroll {
                // selected line top not visible
                self.scroll = hbl;
            } else if hbl + heights[selection] > self.scroll + page_height {
                // selected line bottom not visible
                self.scroll = hbl + heights[selection] - page_height;
            }
        }
    }
    /// Save the drawer, which closes it, then reopen it, keeping the
    /// same state around (scroll and selection)
    pub fn save_and_reopen(
        self,
        open_closet: &mut OpenCloset,
    ) -> Result<Self, SafeClosetError> {
        let DrawerEditState {
            drawer,
            scroll,
            focus,
            search,
            layout,
            ..
        } = self;
        let drawer = open_closet.push_back_save_retake(drawer)?;
        Ok(DrawerEditState {
            drawer,
            scroll,
            focus,
            search,
            edit_count: 0,
            layout,
        })
    }
    pub fn edit_entry_name_by_line(&mut self, line: usize, pos: EditionPos) -> bool {
        if let Some(idx) = self.listed_entry_idx(line) {
            let mut input = ContentSkin::make_input();
            input.set_str(&self.drawer.content.entries[idx].name);
            pos.apply_to_input(&mut input);
            self.focus = DrawerFocus::NameEdit { line, input };
            self.increment_edit_count();
            true
        } else {
            false
        }
    }
    pub fn edit_entry_value_by_line(&mut self, line: usize, pos: EditionPos) -> bool {
        if let Some(idx) = self.listed_entry_idx(line) {
            let mut input = ContentSkin::make_input();
            input.new_line_on(ALT_ENTER);
            input.new_line_on(CONTROL_ENTER);
            input.set_str(&self.drawer.content.entries[idx].value);
            pos.apply_to_input(&mut input);
            self.focus = DrawerFocus::ValueEdit { line, input };
            self.increment_edit_count();
            true
        } else {
            false
        }
    }
    pub fn apply_scroll_command(&mut self, scroll_command: ScrollCommand) {
        self.scroll = scroll_command.apply(self.scroll, self.listed_entries_count(), self.page_height());
    }
    pub fn close_input(&mut self, discard: bool) -> bool {
        if let DrawerFocus::NameEdit { line, input } = &self.focus {
            let line = *line;
            if let Some(idx) = self.listed_entry_idx(line) {
                if discard {
                    self.decrement_edit_count();
                } else {
                    let new_name = input.get_content();
                    if new_name == self.drawer.content.entries[idx].name {
                        self.decrement_edit_count();
                    } else {
                        self.drawer.content.entries[idx].name = new_name;
                    }
                }
                self.focus = DrawerFocus::NameSelected { line };
                return true;
            }
        }
        if let DrawerFocus::ValueEdit { line, input } = &self.focus {
            let line = *line;
            if let Some(idx) = self.listed_entry_idx(line) {
                if discard {
                    self.decrement_edit_count();
                } else {
                    let new_value = input.get_content();
                    if new_value == self.drawer.content.entries[idx].value {
                        self.decrement_edit_count();
                    } else {
                        self.drawer.content.entries[idx].value = new_value;
                    }
                }
                self.focus = DrawerFocus::ValueSelected { line };
                return true;
            }
        }
        if let DrawerFocus::SearchEdit { previous_line } = self.focus {
            self.focus = match self.best_search_line().or(previous_line) {
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
    pub fn has_input(&self) -> bool {
        match &self.focus {
            DrawerFocus::NameEdit { .. } => true,
            DrawerFocus::ValueEdit { .. } => true,
            DrawerFocus::SearchEdit { .. } => true,
            _ => false,
        }
    }
}

