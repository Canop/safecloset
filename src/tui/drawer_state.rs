use {
    super::*,
    crate::{
        core::*,
        error::SafeClosetError,
        search::*,
    },
    crokey::key,
    termimad::{
        Area,
        FmtText,
        InputField,
    },
};

/// State of the application when a drawer is open.
///
/// Contains the open drawer and an entry state.
pub struct DrawerState {
    pub drawer: OpenDrawer,
    pub scroll: usize, // number of listed entries hidden above the top of the view
    pub focus: DrawerFocus,
    edit_count: usize, // a counter to know whether the drawer changed
    pub search: SearchState,
    layout: DrawerDrawingLayout,
}

#[derive(Debug, Clone, Copy)]
pub enum Clicked {
    Nothing,
    Name(usize),
    Value(usize),
    Search,
}

#[derive(Debug, Clone, Copy)]
pub enum EditionPos {
    Start,
    End,
}
impl EditionPos {
    pub fn apply_to_input(
        self,
        input: &mut InputField,
    ) {
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

impl From<OpenDrawer> for DrawerState {
    fn from(drawer: OpenDrawer) -> Self {
        Self {
            drawer,
            scroll: 0,
            focus: DrawerFocus::NoneSelected,
            edit_count: 0,
            search: SearchState::default(),
            layout: DrawerDrawingLayout::default(),
        }
    }
}

impl DrawerState {
    /// Sort entries
    ///
    /// If the list is filtered, only matches are moved
    pub fn sort(&mut self) {
        let entries = &mut self.drawer.content.entries;
        if let Some(result) = self.search.result.as_ref() {
            // We sort among filtered entries, not moving the other ones.
            // Algorithm by @Stargateur
            let matches = &result.entries;
            for (i, m) in matches.iter().enumerate() {
                entries.swap(i, m.idx);
            }
            entries[0..matches.len()].sort_by_key(|e| e.name.to_lowercase());
            for (i, m) in matches.iter().enumerate().rev() {
                entries.swap(i, m.idx);
            }
            self.update_search();
        } else {
            // We sort all entries
            self.focus = DrawerFocus::NoneSelected;
            entries.sort_by_key(|e| e.name.to_lowercase());
        }
    }
    /// Swap the focused line with either the one before or
    /// the one after.
    /// Depending on the focus, this "line" is either an entry, or
    /// a text line in the edited value
    pub fn swap_line(
        &mut self,
        dir: Direction,
    ) {
        use DrawerFocus::*;
        if let ValueEdit { input, .. } = &mut self.focus {
            match dir {
                Direction::Up => input.move_current_line_up(),
                Direction::Down => input.move_current_line_down(),
            };
        } else {
            let Some(line) = self.focus.line() else {
                return;
            };
            let entries = &mut self.drawer.content.entries;
            let matches = self.search.result.as_ref().map(|r| &r.entries);
            let len = if let Some(matches) = matches {
                matches.len()
            } else {
                entries.len()
            };
            if len < 2 {
                return;
            }
            let new_line = match dir {
                Direction::Up => (line + len - 1) % len,
                Direction::Down => (line + len + 1) % len,
            };
            if let Some(matches) = matches {
                // we convert (line, new_line) into real entry indexes
                entries.swap(matches[line].idx, matches[new_line].idx);
            } else {
                // directly swapping entries, it's easy
                entries.swap(line, new_line);
            };
            if let Some(line) = self.focus.selection_mut() {
                *line = new_line
            }
        }
        self.update_search();
    }
    /// Move the focus/selection one line up
    pub fn move_line(
        &mut self,
        dir: Direction,
    ) {
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

    /// Move entries so that the matching ones are together
    /// (entries up to and including the first matching one
    /// don't move).
    /// Order among matches, and order among non-matches, are
    /// preserved.
    /// Search is cleared and focus is set to selection of the group's head
    pub fn group_matching_entries(&mut self) {
        let matches = self.search.result.as_mut().map(|r| &mut r.entries);
        let Some(matches) = matches else { return };
        // head index where we stack all matches
        let Some(head) = matches.first().map(|m| m.idx) else {
            return;
        };
        if matches.is_empty() {
            return;
        };
        let entries = &mut self.drawer.content.entries;
        let mut ordered_entries: Vec<_> = entries.drain(0..head).collect();
        for m in matches {
            m.idx -= ordered_entries.len();
            ordered_entries.push(entries.remove(m.idx));
        }
        ordered_entries.append(entries);
        self.drawer.content.entries = ordered_entries;
        self.search.clear();
        self.focus = DrawerFocus::NameSelected { line: head };
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
        debug_assert!(content_area.height > 4);
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
        let max_value_height = if page_height > 7 {
            page_height as usize - 5
        } else {
            page_height as usize - 2
        };
        for l in 0..lines_count {
            let idx = self.listed_entry_idx(l).unwrap(); // SAFETY: we iter among valid lines
            let height = match &self.focus {
                DrawerFocus::ValueEdit { input, line } if l == *line => {
                    // this line's value is edited, its height is given by the
                    // number of lines computed by the input
                    input.content().line_count().min(max_value_height)
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
                        text.lines.len().min(max_value_height).max(1)
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

    pub fn values_as_markdown(&self) -> bool {
        self.drawer.content.settings.values_as_markdown
    }

    pub fn update_search(&mut self) {
        self.search.update(&self.drawer)
    }

    /// Tell what part of the drawer screen has been clicked
    pub fn clicked(
        &self,
        x: u16,
        y: usize,
    ) -> Clicked {
        let in_name_col = self.layout.is_in_name_column(x);
        let mut sum_heights = self.layout.lines_area.top as usize;
        if y < sum_heights {
            if in_name_col && y > 1 {
                return Clicked::Search;
            }
        } else {
            let heights = self
                .layout
                .value_heights_by_line
                .iter()
                .enumerate()
                .skip(self.scroll);
            for (line, height) in heights {
                sum_heights += height;
                if sum_heights > y {
                    if in_name_col {
                        return Clicked::Name(line);
                    } else {
                        return Clicked::Value(line);
                    }
                }
            }
        }
        Clicked::Nothing
    }

    pub fn scrollbar(&self) -> Option<(u16, u16)> {
        let content_height = self.layout.content_height;
        let page_height = self.page_height();
        if page_height >= content_height {
            return None;
        }
        let heights = &self.layout.value_heights_by_line;
        let real_hidden_before = heights[0..self.scroll].iter().sum::<usize>() as f64;
        let real_from_scroll = heights[self.scroll..].iter().sum::<usize>() as f64;
        let real_hidden_after = real_from_scroll - page_height as f64;
        let hidden_before = if real_hidden_before > 0.0 {
            (page_height as u16 * real_hidden_before as u16 / content_height as u16).max(1)
        } else {
            0
        };
        let hidden_after = if real_hidden_after > 0.0 {
            (page_height as u16 * real_hidden_after as u16 / content_height as u16).max(1)
        } else {
            0
        };
        Some((
            self.layout.lines_area.top + hidden_before,
            self.layout.lines_area.bottom() - hidden_after,
        ))
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
            NameSelected { line } | NameEdit { line, .. } => self
                .listed_entry_idx(*line)
                .and_then(|idx| self.drawer.content.entries.get(idx))
                .map(|entry| entry.name.as_str()),
            ValueSelected { line } | ValueEdit { line, .. } => self
                .listed_entry_idx(*line)
                .and_then(|idx| self.drawer.content.entries.get(idx))
                .map(|entry| entry.value.as_str()),
            _ => None,
        }
    }

    pub fn entry_line(
        &self,
        idx: usize,
    ) -> Option<usize> {
        if let Some(search_result) = &self.search.result {
            for (line, matching_entry) in search_result.entries.iter().enumerate() {
                if matching_entry.idx == idx {
                    return Some(line);
                }
            }
        } else if idx < self.drawer.content.entries.len() {
            return Some(idx);
        }
        None
    }

    /// Give the index of the entry from its line among the listed
    /// entries (either all entries or only the ones matching if there's
    /// a search)
    pub fn listed_entry_idx(
        &self,
        line: usize,
    ) -> Option<usize> {
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
    pub fn listed_entry(
        &self,
        line: usize,
    ) -> Option<(usize, Option<NameMatch>)> {
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
                        self.listed_entry_idx(*line).map_or(true, |idx| {
                            !input.is_content(&self.drawer.content.entries[idx].name)
                        })
                    }
                    DrawerFocus::ValueEdit { line, input } => {
                        self.listed_entry_idx(*line).map_or(true, |idx| {
                            !input.is_content(&self.drawer.content.entries[idx].value)
                        })
                    }
                    _ => true,
                }
            }
            _ => true,
        }
    }
    pub fn apply_scroll_command(
        &mut self,
        scroll_command: ScrollCommand,
    ) {
        let page_height = self.page_height();
        let initial_scroll = self.scroll;
        self.scroll = scroll_command.apply(self.scroll, self.layout.content_height, page_height);
        // if fix_scroll reverts to the previous position, it's because doing
        // differently would hide the selection. In this case, and when the
        // selection can be moved, we move it (and the fix_scroll happening
        // at the start of the drawing will make it visible).
        self.fix_scroll();
        if self.scroll == initial_scroll {
            let count = self.listed_entries_count();
            let last_visible_line = self.last_visible_line();
            if let Some(selection) = self.focus.selection_mut() {
                match scroll_command {
                    ScrollCommand::Top => {
                        *selection = 0;
                    }
                    ScrollCommand::Bottom => {
                        // safety: lines_count > 0 or we would not have any selection
                        *selection = count - 1;
                    }
                    ScrollCommand::Lines(i) if i < 0 => {
                        if *selection > 0 {
                            *selection -= 1;
                        }
                    }
                    ScrollCommand::Lines(i) if i > 0 => {
                        if *selection + 1 < count {
                            *selection += 1;
                        }
                    }
                    ScrollCommand::Pages(-1) => {
                        *selection = initial_scroll;
                    }
                    ScrollCommand::Pages(1) => {
                        // safety: there's a last visible line or there would be not selection
                        *selection = last_visible_line.unwrap();
                    }
                    _ => {
                        warn!("unacounted scroll command: {:?}", scroll_command);
                    }
                }
            }
        }
    }
    pub fn last_visible_line(&self) -> Option<usize> {
        let page_height = self.page_height();
        let heights = self
            .layout
            .value_heights_by_line
            .iter()
            .enumerate()
            .skip(self.scroll);
        let mut sum_heights = 0;
        for (line, height) in heights {
            sum_heights += height;
            if sum_heights > page_height {
                return Some(line);
            }
        }
        let count = self.listed_entries_count();
        if count > 0 { Some(count - 1) } else { None }
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
        if self.scroll >= content_height {
            // this may happen after searching
            self.scroll = 0;
        }
        if content_height <= page_height {
            self.scroll = 0;
        } else if let Some(selection) = self.focus.line() {
            if self.scroll >= selection {
                self.scroll = selection;
            } else {
                // let's ensure the end of the selection is visible
                while heights[self.scroll..=selection].iter().sum::<usize>() > page_height {
                    self.scroll += 1;
                }
                // let's ensure there's not too much void at end
                let last_entry = self.listed_entries_count() - 1;
                while self.scroll > selection
                    && heights[self.scroll - 1..=last_entry].iter().sum::<usize>() >= page_height
                {
                    self.scroll -= 1;
                }
            }
        }
    }
    /// Save the drawer, which closes it, then reopen it, keeping the
    /// same state around (scroll and selection)
    pub fn save_and_reopen(
        self,
        open_closet: &mut OpenCloset,
    ) -> Result<Self, SafeClosetError> {
        let DrawerState {
            drawer,
            scroll,
            focus,
            search,
            layout,
            ..
        } = self;
        let drawer = open_closet.push_back_save_retake(drawer)?;
        Ok(DrawerState {
            drawer,
            scroll,
            focus,
            search,
            edit_count: 0,
            layout,
        })
    }
    pub fn edit_entry_name_by_line(
        &mut self,
        line: usize,
        pos: EditionPos,
    ) -> bool {
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
    pub fn edit_entry_value_by_line(
        &mut self,
        line: usize,
        pos: EditionPos,
    ) -> bool {
        if let Some(idx) = self.listed_entry_idx(line) {
            let mut input = ContentSkin::make_input();
            input.new_line_on(key!(alt - enter));
            input.new_line_on(key!(ctrl - enter));
            input.set_str(&self.drawer.content.entries[idx].value);
            pos.apply_to_input(&mut input);
            self.focus = DrawerFocus::ValueEdit { line, input };
            self.increment_edit_count();
            true
        } else {
            false
        }
    }
    pub fn close_input(
        &mut self,
        discard: bool,
    ) -> bool {
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
        if let DrawerFocus::SearchEdit { previous_idx } = self.focus {
            if discard {
                // FIXME be back to previous focus ?
                self.search.clear();
            }
            self.search.update(&self.drawer);
            self.focus = self
                .best_search_line()
                .or_else(|| previous_idx.and_then(|idx| self.entry_line(idx)))
                .map_or(DrawerFocus::NoneSelected, |line| {
                    DrawerFocus::NameSelected { line }
                });
            return true;
        }
        false
    }
    pub fn has_best_search(
        &self,
        line: usize,
    ) -> bool {
        self.best_search_line().map_or(false, |l| l == line)
    }
    pub fn best_search_line(&self) -> Option<usize> {
        if self.focus.is_search() {
            self.search.result.as_ref().and_then(|r| r.best_line)
        } else {
            None
        }
    }
    /// Return the number of filtered entries (0 if there's no search)
    pub fn match_count(&self) -> usize {
        self.search.result.as_ref().map_or(0, |r| r.entries.len())
    }
}
