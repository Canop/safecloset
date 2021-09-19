use {
    super::*,
    crate::{core::*, error::SafeClosetError},
    crossterm::{self, event::KeyEvent},
};

/// TUI Application state, containing a drawer state.
///
/// Needs a closet
pub struct AppState {
    pub open_closet: OpenCloset,
    pub drawer_state: DrawerState,
    pub error: Option<String>,
    pub hide_values: bool,
    // number of drawers created during this session
    pub created_drawers: usize,
}

impl AppState {

    pub fn new(open_closet: OpenCloset, hide_values: bool) -> Self {
        Self {
            open_closet,
            drawer_state: DrawerState::NoneOpen,
            error: None,
            hide_values,
            created_drawers: 0,
        }
    }

    /// If there's an open drawer input (entry name or value), close it, keeping
    /// the input content if required.
    ///
    /// Return true if there was such input
    fn close_drawer_input(&mut self, discard: bool) -> bool {
        if let DrawerState::DrawerEdit(des) = &mut self.drawer_state {
            des.close_input(discard)
        } else {
            false
        }
    }

    /// Save the content of the edited cell if any, then save the whole closet
    fn save(&mut self, reopen_if_open: bool) -> Result<(), SafeClosetError> {
        time!(self.close_drawer_input(false));
        let drawer_state = std::mem::take(&mut self.drawer_state);
        if let DrawerState::DrawerEdit(des) = drawer_state {
            if reopen_if_open {
                self.drawer_state = DrawerState::DrawerEdit(
                    time!(des.save_and_reopen(&mut self.open_closet)?)
                );
            } else {
                time!(self.open_closet.push_back(des.drawer)?);
                time!(self.open_closet.close_and_save())?;
            }
        }
        Ok(())
    }

    /// Handle a click event
    pub fn on_click(&mut self, x: u16, y: u16)-> Result<(), SafeClosetError> {

        // TODO handle click in search input location

        if let Some(input) = self.drawer_state.input() {
            if input.apply_click_event(x, y) {
                return Ok(());
            } else if let DrawerState::DrawerEdit(des) = &mut self.drawer_state {
                // unfocusing the input, validating it
                debug!("unfocusing des input");
                des.focus = DrawerFocus::NoneSelected;
            }
        }

        if let DrawerState::DrawerEdit(des) = &mut self.drawer_state {
            if let Some(clicked_line) = des.clicked_line(y) {
                use DrawerFocus::*;
                let in_name = des.layout().is_in_name_column(x);
                des.focus = if in_name {
                    NameSelected { line: clicked_line }
                } else {
                    ValueSelected { line: clicked_line }
                };
            }
        }

        Ok(())
    }

    /// push back the open drawer, if any, and set the drawer_state to NoneOpen
    fn push_back_drawer(&mut self) -> Result<(), SafeClosetError> {
        self.close_drawer_input(true);
        // if there's an edited drawer, we push it back to the closet
        let drawer_state = std::mem::take(&mut self.drawer_state);
        if let DrawerState::DrawerEdit(DrawerEditState { drawer, .. }) = drawer_state {
            self.open_closet.push_back(drawer)?;
        }
        Ok(())
    }

    /// Handle a key event
    pub fn on_key(&mut self, key: KeyEvent) -> Result<CmdResult, SafeClosetError> {
        use {
            DrawerFocus::*,
            DrawerState::*,
        };
        self.error = None;

        if key == CONTROL_C { // close drawer (no save)
            // we're not repushing the drawer, so we're effectively
            // going up in the closet
            self.drawer_state = DrawerState::NoneOpen;
            return Ok(CmdResult::Stay);
        }

        if key == CONTROL_N { // new drawer
            self.push_back_drawer()?;
            self.drawer_state = DrawerCreation(PasswordInputState::new(false));
            return Ok(CmdResult::Stay);
        }

        if key == CONTROL_O { // open drawer
            self.push_back_drawer()?;
            self.drawer_state = DrawerOpening(PasswordInputState::new(true));
            return Ok(CmdResult::Stay);
        }

        if key == CONTROL_Q {
            debug!("user requests quit");
            return Ok(CmdResult::Quit);
        }

        if key == CONTROL_S {
            debug!("user requests save, keep state");
            self.save(true)?;
            return Ok(CmdResult::Stay);
        }

        if key == CONTROL_X {
            debug!("user requests save and quit");
            self.save(false)?;
            return Ok(CmdResult::Quit);
        }

        if key == CONTROL_UP { // moving the selected line up
            if let DrawerEdit(des) = &mut self.drawer_state {
                des.close_input(false);
                let entries = &mut des.drawer.content.entries;
                let len = entries.len();
                match des.focus {
                    NameSelected { line } => {
                        let new_line = (line + len - 1) % len;
                        entries.swap(line, new_line);
                        des.focus = NameSelected { line: new_line };
                    }
                    ValueSelected { line } => {
                        let new_line = (line + len - 1) % len;
                        entries.swap(line, new_line);
                        des.focus = ValueSelected { line: new_line };
                    }
                    _ => {}
                }
            }
            return Ok(CmdResult::Stay);
        }
        if key == CONTROL_DOWN { // moving the selected line down
            if let DrawerEdit(des) = &mut self.drawer_state {
                des.close_input(false);
                let entries = &mut des.drawer.content.entries;
                let len = entries.len();
                match des.focus {
                    NameSelected { line } => {
                        let new_line = (line + 1) % len;
                        entries.swap(line, new_line);
                        des.focus = NameSelected { line: new_line };
                    }
                    ValueSelected { line } => {
                        let new_line = (line + 1) % len;
                        entries.swap(line, new_line);
                        des.focus = ValueSelected { line: new_line };
                    }
                    _ => {}
                }
            }
            return Ok(CmdResult::Stay);
        }


        if let DrawerEdit(des) = &mut self.drawer_state {
            // -- pending removal
            if let PendingRemoval { line } = &des.focus {
                let line = *line;
                if let Some(idx) = des.listed_entry_idx(line) {
                    // we either confirm (delete) or cancel removal
                    if as_letter(key) == Some('y') {
                        info!("user requests entry removal");
                        des.drawer.content.entries.remove(idx);
                        des.focus = if line > 0 {
                            NameSelected { line }
                        } else {
                            NoneSelected
                        };
                    } else {
                        info!("user cancels entry removal");
                        des.focus = NameSelected { line };
                    }
                }
                return Ok(CmdResult::Stay);
            }
        }

        // -- toggle visibility of password or values

        if key == CONTROL_H {
            if let DrawerCreation(pis) | DrawerOpening(pis) = &mut self.drawer_state {
                pis.input.password_mode ^= true;
                return Ok(CmdResult::Stay);
            }
            if let DrawerEdit(des) = &mut self.drawer_state {
                des.drawer.content.settings.hide_values ^= true;
                return Ok(CmdResult::Stay);
            }
        }

        // --

        if key == ENTER {
            self.close_drawer_input(false); // if there's an entry input
            if let DrawerCreation(PasswordInputState { input }) = &mut self.drawer_state {
                let pwd = input.get_content();
                let open_drawer = time!(self.open_closet.create_take_drawer(&pwd));
                match open_drawer {
                    Ok(open_drawer) => {
                        self.drawer_state = DrawerState::edit(open_drawer);
                        self.created_drawers += 1;
                    }
                    Err(e) => {
                        warn!("error in drawer creation: {}", e);
                        self.error = Some(e.to_string());
                    }
                }
            } else if let DrawerOpening(PasswordInputState { input }) = &mut self.drawer_state {
                let pwd = input.get_content();
                let open_drawer = self.open_closet.open_take_drawer(&pwd);
                match open_drawer {
                    Some(mut open_drawer) => {
                        if self.hide_values {
                            open_drawer.content.settings.hide_values = true;
                        }
                        self.drawer_state = DrawerState::edit(open_drawer);
                    }
                    None => {
                        warn!("no drawer can be opened with this passphrase");
                        self.error = Some("This passphrase opens no drawer".to_string());
                    }
                }
            }
            return Ok(CmdResult::Stay);
        }

        if key == ESC {
            if matches!(self.drawer_state, DrawerCreation(_) | DrawerOpening(_)) {
                self.drawer_state = NoneOpen;
            } else if let DrawerEdit(des) = &mut self.drawer_state {
                if !des.close_input(true) {
                    des.focus = NoneSelected;
                }
            }
            return Ok(CmdResult::Stay);
        }

        if key == TAB {
            if let DrawerEdit(des) = &mut self.drawer_state {
                if matches!(des.focus, NoneSelected) {
                    // we remove any search
                    des.search.clear();
                    let idx = des.drawer.content.empty_entry();
                    des.edit_entry_name_by_line(idx); // as there's no filtering, idx==line
                } else if let NameSelected { line } = &des.focus {
                    let line = *line;
                    des.edit_entry_value_by_line(line);
                } else if let NameEdit { line, .. } = &des.focus {
                    let line = *line;
                    des.close_input(false);
                    des.edit_entry_value_by_line(line);
                } else if let ValueSelected { line } | ValueEdit { line, .. } = &des.focus {
                    let line = *line;
                    if des.listed_entries_count() == line + 1 {
                        // last listed entry
                        if des.drawer.content.entries[line].is_empty() {
                            // if the current entry is empty, we don't create a new one
                            // but go back to the current (empty) entry name
                            des.edit_entry_name_by_line(line);
                        } else {
                            // we create a new entry and start edit it
                            // but we must ensure there's no search which could filter it
                            des.search.clear();
                            des.drawer.content.entries.push(Entry::default());
                            des.edit_entry_name_by_line(des.drawer.content.entries.len() - 1);
                        }
                    } else {
                        des.edit_entry_name_by_line(line + 1);
                    }
                }
                return Ok(CmdResult::Stay);
            }
        }

        // --- input

        if let Some(input) = self.drawer_state.input() {
            if input.apply_key_event(key) {
                if let DrawerEdit(des) = &mut self.drawer_state {
                    if des.focus.is_search() {
                        des.search.update(&des.drawer);
                    }
                }
                return Ok(CmdResult::Stay);
            }
        }

        // --- navigation among entries

        if let DrawerEdit(des) = &mut self.drawer_state {
            if key == HOME {
                des.apply_scroll_command(ScrollCommand::Top);
                return Ok(CmdResult::Stay);
            }
            if key == END {
                des.apply_scroll_command(ScrollCommand::Bottom);
                return Ok(CmdResult::Stay);
            }
            if key == PAGE_UP {
                des.apply_scroll_command(ScrollCommand::Pages(-1));
                return Ok(CmdResult::Stay);
            }
            if key == PAGE_DOWN {
                des.apply_scroll_command(ScrollCommand::Pages(1));
                return Ok(CmdResult::Stay);
            }
        }

        if key == INSERT || as_letter(key) == Some('i') {
            if let DrawerEdit(des) = &mut self.drawer_state {
                if let NameSelected { line } = &des.focus {
                    let line = *line;
                    des.edit_entry_name_by_line(line);
                }
                if let ValueSelected { line } = &des.focus {
                    let line = *line;
                    des.edit_entry_value_by_line(line);
                }
            }
            return Ok(CmdResult::Stay);
        }

        if let DrawerEdit(des) = &mut self.drawer_state {
            if key == RIGHT {
                if let NameSelected { line } = &des.focus {
                    let line = *line;
                    des.focus = ValueSelected { line };
                }
                return Ok(CmdResult::Stay);
            }
            if key == LEFT {
                if let ValueSelected { line } = &des.focus {
                    let line = *line;
                    des.focus = NameSelected { line };
                }
                return Ok(CmdResult::Stay);
            }
            if key == UP {
                if des.focus.is_search() {
                    if let Some(line) = des.best_search_line() {
                        let line = if line > 0 {
                            line - 1
                        } else {
                            des.listed_entries_count() - 1
                        };
                        des.focus = NameSelected { line };
                    } else {
                        // there's no match, so there's no point to keep the search
                        des.search.clear();
                        des.search.update(&des.drawer);
                        des.focus = NameSelected { line: 0 };
                    }
                    return Ok(CmdResult::Stay);
                }
                if let NameSelected { line } = &des.focus {
                    let line = if *line > 0 {
                        line - 1
                    } else {
                        des.listed_entries_count() - 1
                    };
                    des.focus = NameSelected { line };
                }
                if let ValueSelected { line } = &des.focus {
                    let line = if *line > 0 {
                        line - 1
                    } else {
                        des.listed_entries_count() - 1
                    };
                    des.focus = ValueSelected { line };
                }
                if matches!(des.focus, NoneSelected) {
                    des.focus = NameSelected { line: 0 };
                }
                return Ok(CmdResult::Stay);
            }
            if key == DOWN {
                if des.focus.is_search() {
                    if let Some(line) = des.best_search_line() {
                        let line = if line < des.listed_entries_count() {
                            line + 1
                        } else {
                            0
                        };
                        des.focus = NameSelected { line };
                    } else {
                        // there's no match, so there's no point to keep the search
                        des.search.clear();
                        des.search.update(&des.drawer);
                        des.focus = NameSelected { line: 0 };
                    }
                    return Ok(CmdResult::Stay);
                }
                if let NameSelected { line } = &des.focus {
                    let line = if *line + 1 < des.listed_entries_count() {
                        line + 1
                    } else {
                        0
                    };
                    des.focus = NameSelected { line };
                }
                if let ValueSelected { line } = &des.focus {
                    let line = if *line + 1 < des.listed_entries_count() {
                        line + 1
                    } else {
                        0
                    };
                    des.focus = ValueSelected { line };
                }
                if matches!(des.focus, NoneSelected) {
                    des.focus = NameSelected { line: 0 };
                }
                return Ok(CmdResult::Stay);
            }
        }

        // --- other simple char shortcuts

        if let Some(letter) = as_letter(key) {

            if let DrawerEdit(des) = &mut self.drawer_state {
                // if we're here, there's no input
                match (letter, des.focus.line()) {
                    ('n', _) => {
                        // new entry
                        des.search.clear();
                        let idx = des.drawer.content.empty_entry();
                        des.edit_entry_name_by_line(idx);
                    }
                    ('d', Some(line)) => {
                        // delete entry (with confirmation)
                        des.focus = PendingRemoval { line };
                    }
                    ('/', _) => {
                        // start searching
                        des.focus = SearchEdit;
                    }
                    _ => {}
                }
                return Ok(CmdResult::Stay);
            }
        }

        Ok(CmdResult::Stay)
    }
}
