use {
    super::*,
    crate::{
        cli::Args,
        core::*,
        error::SafeClosetError,
    },
    crossterm::event::{
        KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
};

/// TUI Application state
pub struct AppState {
    pub open_closet: OpenCloset,
    pub drawer_state: DrawerState,
    // the help state, if help is currently displayed
    pub help: Option<HelpState>,
    // the menu, if any
    pub menu: Option<MenuState>,
    pub message: Option<Message>,
    pub hide_values: bool,
    // number of drawers created during this session
    pub created_drawers: usize,
}

impl AppState {

    pub fn new(open_closet: OpenCloset, args: &Args) -> Self {
        let mut state = Self {
            open_closet,
            drawer_state: DrawerState::NoneOpen,
            help: None,
            menu: None,
            message: None,
            hide_values: args.hide,
            created_drawers: 0,
        };
        if args.open && !state.open_closet.just_created() {
            state.drawer_state = DrawerState::DrawerOpening(PasswordInputState::new(true));
        }
        state
    }

    fn set_error<S: Into<String>>(&mut self, error: S) {
        let text = error.into();
        warn!("error: {:?}", &text);
        self.message = Some(Message{ text, error: true });
    }
    #[allow(dead_code)]
    fn set_info<S: Into<String>>(&mut self, info: S) {
        let text = info.into();
        debug!("info: {:?}", &text);
        self.message = Some(Message{ text, error: false });
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
        if let DrawerState::DrawerEdit(mut des) = drawer_state {
            if reopen_if_open {
                self.drawer_state = DrawerState::DrawerEdit(
                    time!(des.save_and_reopen(&mut self.open_closet)?)
                );
            } else {
                des.drawer.content.remove_empty_entries();
                time!(self.open_closet.push_back(des.drawer)?);
                time!(self.open_closet.close_and_save())?;
            }
        }
        Ok(())
    }

    /// Handle an event asking for copying from SafeCloset
    pub fn copy(&mut self) {
        #[cfg(not(feature = "clipboard"))]
        {
            self.set_error("Clipboard feature not enabled at compilation");
        }
        #[cfg(feature = "clipboard")]
        {
            if let Some(input) = self.drawer_state.input() {
                let s = input.copy_selection();
                if let Err(e) = terminal_clipboard::set_string(&s) {
                    self.set_error(e.to_string());
                } else if !s.is_empty() {
                    self.set_info("string copied to the clipboard, be cautious");
                }
            } else if let DrawerState::DrawerEdit(des) = &self.drawer_state {
                if let Some(cell) = des.current_cell() {
                    if let Err(e) = terminal_clipboard::set_string(cell) {
                        self.set_error(e.to_string());
                    } else {
                        self.set_info("cell copied to the clipboard, be cautious");
                    }
                } else {
                    self.set_error("you can only copy from a selected name or value");
                }
            } else {
                self.set_error("you can only copy from an open drawer");
            }
        }
    }

    /// Handle an event asking for cutting from SafeCloset
    pub fn cut(&mut self) {
        #[cfg(not(feature = "clipboard"))]
        {
            self.set_error("Clipboard feature not enabled at compilation");
        }
        #[cfg(feature = "clipboard")]
        {
            if let Some(input) = self.drawer_state.input() {
                let s = input.cut_selection();
                if let Err(e) = terminal_clipboard::set_string(&s) {
                    self.set_error(e.to_string());
                } else if !s.is_empty() {
                    self.set_info("string copied to the clipboard, be cautious");
                }
            } else {
                self.set_error("you can only copy from an edited input");
            }
        }
    }

    /// Handle an event asking for pasting into SafeCloset
    pub fn paste(&mut self) {
        #[cfg(not(feature = "clipboard"))]
        {
            self.set_error("Clipboard feature not enabled at compilation");
        }
        #[cfg(feature = "clipboard")]
        {
            use DrawerFocus::*;
            match terminal_clipboard::get_string() {
                Ok(mut pasted) if !pasted.is_empty() => {
                    if !self.drawer_state.is_on_entry_value() {
                        pasted.truncate(pasted.lines().next().unwrap().len());
                    }
                    if let Some(input) = self.drawer_state.input() {
                        input.replace_selection(pasted);
                    } else if let DrawerState::DrawerEdit(des) = &mut self.drawer_state {
                        if let NameSelected { line } = &mut des.focus {
                            let line = *line;
                            if des.edit_entry_name_by_line(line, EditionPos::Start) {
                                if let Some(input) = self.drawer_state.input() {
                                    input.set_str(pasted);
                                    input.move_to_end();
                                    self.set_info("Hit *esc* to cancel pasting");
                                } else {
                                    warn!("unexpected lack of input");
                                }
                            }
                        } else if let ValueSelected { line } = &mut des.focus {
                            let line = *line;
                            if des.edit_entry_value_by_line(line, EditionPos::Start) {
                                if let Some(input) = self.drawer_state.input() {
                                    input.set_str(pasted);
                                    input.move_to_end();
                                    self.set_info("Hit *esc* to cancel pasting");
                                } else {
                                    warn!("unexpected lack of input");
                                }
                            }
                        }
                    }
                }
                _ => {
                    self.set_error("nothing to paste");
                }
            }
        }
    }

    pub fn on_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        double_click: bool,
    )-> Result<(), SafeClosetError> {

        // TODO handle click in search input location

        if let Some(input) = self.drawer_state.input() {
            if input.apply_mouse_event(mouse_event, double_click) {
                return Ok(());
            } else if let DrawerState::DrawerEdit(des) = &mut self.drawer_state {
                // unfocusing the input, validating it
                des.focus = DrawerFocus::NoneSelected;
            }
        }
        if let DrawerState::DrawerEdit(des) = &mut self.drawer_state {
            let MouseEvent {
                kind,
                row, column,
                modifiers,
            } = mouse_event;
            match kind {
                MouseEventKind::Up(MouseButton::Left) => {
                    if modifiers == KeyModifiers::NONE {
                        if let Some(clicked_line) = des.clicked_line(row as usize) {
                            use DrawerFocus::*;
                            // if we're here we know the clicked input isn't focused
                            let in_name = des.layout().is_in_name_column(column);
                            if in_name {
                                if des.focus.is_name_selected(clicked_line) {
                                    des.edit_entry_name_by_line(clicked_line, EditionPos::Start);
                                } else {
                                    des.focus = NameSelected { line: clicked_line };
                                }
                            } else {
                                if des.focus.is_value_selected(clicked_line) {
                                    des.edit_entry_value_by_line(clicked_line, EditionPos::Start);
                                } else {
                                    des.focus = ValueSelected { line: clicked_line };
                                }

                            }
                        }
                    }
                }
                MouseEventKind::ScrollUp => {
                    des.move_line(Direction::Up);
                }
                MouseEventKind::ScrollDown => {
                    des.move_line(Direction::Down);
                }
                _ => {}
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

    /// delete entry (with confirmation)
    fn propose_entry_removal(&mut self) {
        if let DrawerState::DrawerEdit(des) = &mut self.drawer_state {
            if let Some(line) = des.focus.line() {
                des.focus = DrawerFocus::PendingRemoval { line };
                let mut menu = MenuState::default();
                menu.actions.push(Action::ConfirmEntryRemoval);
                menu.actions.push(Action::Back);
                menu.selection = 1;
                self.menu = Some(menu);
            }
        }
    }
    fn cancel_entry_removal(&mut self) {
        if let DrawerState::DrawerEdit(des) = &mut self.drawer_state {
            if let DrawerFocus::PendingRemoval { line } = &des.focus {
                let line = *line;
                des.focus = DrawerFocus::NameSelected { line };
                self.menu = None;
                self.help = None;
            }
        }
    }

    pub fn on_action(&mut self, action: Action) -> Result<CmdResult, SafeClosetError> {
        use {
            DrawerFocus::*,
            DrawerState::*,
        };
        debug!("executing action {:?}", action);
        match action {
            Action::Back => {
                if self.drawer_state.is_pending_removal() {
                    self.cancel_entry_removal();
                } else if self.menu.is_some() {
                    self.menu = None;
                } else if self.help.is_some() {
                    self.help = None;
                } else if matches!(self.drawer_state, DrawerCreation(_) | DrawerOpening(_)) {
                    self.drawer_state = NoneOpen;
                } else if self.close_drawer_input(true) {
                    debug!("closing drawer input");
                } else {
                    debug!("opening menu");
                    let mut menu = MenuState::default();
                    self.fill_menu(&mut menu.actions);
                    self.menu = Some(menu);
                }
            }
            Action::NewDrawer => {
                self.help = None;
                self.menu = None;
                self.push_back_drawer()?;
                self.drawer_state = DrawerCreation(PasswordInputState::new(false));
            }
            Action::OpenDrawer => {
                self.help = None;
                self.menu = None;
                self.push_back_drawer()?;
                self.drawer_state = DrawerOpening(PasswordInputState::new(true));
            }
            Action::SaveDrawer => {
                if self.drawer_state.is_edit() {
                    self.help = None;
                    self.menu = None;
                    debug!("user requests save, keep state");
                    self.save(true)?;
                } else {
                    self.set_error("no open drawer");
                }
            }
            Action::CloseDrawer => {
                self.help = None;
                self.menu = None;
                self.save(true)?;
                self.push_back_drawer()?;
                let _ = self.open_closet.close_deepest_drawer();
                self.drawer_state = match self.open_closet.take_deepest_open_drawer() {
                    Some(open_drawer) => DrawerState::edit(open_drawer),
                    None => DrawerState::NoneOpen,
                };
            }
            Action::Help => {
                self.help = Some(HelpState::default());
                self.menu = None;
            }
            Action::Quit => {
                debug!("user requests quit");
                return Ok(CmdResult::Quit);
            }
            Action::MoveLineUp => {
                if let DrawerEdit(des) = &mut self.drawer_state {
                    let entries = &mut des.drawer.content.entries;
                    let len = entries.len();
                    match &mut des.focus {
                        NameSelected { line } => {
                            let new_line = (*line + len - 1) % len;
                            entries.swap(*line, new_line);
                            des.focus = NameSelected { line: new_line };
                        }
                        ValueSelected { line } => {
                            let new_line = (*line + len - 1) % len;
                            entries.swap(*line, new_line);
                            des.focus = ValueSelected { line: new_line };
                        }
                        ValueEdit { input, .. }  => {
                            input.move_current_line_up();
                        }
                        _ => {}
                    }
                    des.update_search();
                }
            }
            Action::MoveLineDown => {
                if let DrawerEdit(des) = &mut self.drawer_state {
                    let entries = &mut des.drawer.content.entries;
                    let len = entries.len();
                    match &mut des.focus {
                        NameSelected { line } => {
                            let new_line = (*line + 1) % len;
                            entries.swap(*line, new_line);
                            des.focus = NameSelected { line: new_line };
                        }
                        ValueSelected { line } => {
                            let new_line = (*line + 1) % len;
                            entries.swap(*line, new_line);
                            des.focus = ValueSelected { line: new_line };
                        }
                        ValueEdit { input, .. }  => {
                            input.move_current_line_down();
                        }
                        _ => {}
                    }
                    des.update_search();
                }
            }
            Action::ToggleHiding => {
                // toggle visibility of password or values
                self.help = None;
                self.menu = None;
                if let DrawerCreation(pis) | DrawerOpening(pis) = &mut self.drawer_state {
                    pis.input.password_mode ^= true;
                    return Ok(CmdResult::Stay);
                }
                if let DrawerEdit(des) = &mut self.drawer_state {
                    des.drawer.content.settings.hide_values ^= true;
                    return Ok(CmdResult::Stay);
                }
            }
            Action::OpenAllValues | Action::CloseAllValues=> {
                self.help = None;
                self.menu = None;
                if let DrawerEdit(des) = &mut self.drawer_state {
                    des.drawer.content.settings.open_all_values ^= true;
                    return Ok(CmdResult::Stay);
                }
            }
            Action::Copy => {
                self.help = None;
                self.menu = None;
                self.copy();
            }
            Action::Cut => {
                self.help = None;
                self.menu = None;
                self.cut();
            }
            Action::Paste => {
                self.help = None;
                self.menu = None;
                self.paste();
            }
            Action::ConfirmEntryRemoval => {
                self.help = None;
                self.menu = None;
                info!("user requests entry removal");
                if let DrawerEdit(des) = &mut self.drawer_state {
                    if let PendingRemoval { line } = &des.focus {
                        let line = *line;
                        if let Some(idx) = des.listed_entry_idx(line) {
                            // we either confirm (delete) or cancel removal
                            des.drawer.content.entries.remove(idx);
                            des.focus = if line > 0 {
                                NameSelected { line: line - 1 }
                            } else {
                                NoneSelected
                            };
                            des.update_search();
                        }
                    }
                }
            }
            Action::NewEntry => {
                if let DrawerEdit(des) = &mut self.drawer_state {
                    self.help = None;
                    self.menu = None;
                    des.search.clear();
                    let idx = des.drawer.content.empty_entry();
                    des.edit_entry_name_by_line(idx, EditionPos::Start);
                }
            }
            Action::RemoveLine => {
                self.propose_entry_removal();
            }
            Action::Search => {
                if let DrawerEdit(des) = &mut self.drawer_state {
                    des.focus = SearchEdit { previous_line: des.focus.line() };
                }
            }
        }
        Ok(CmdResult::Stay)
    }

    /// Add the relevant possible actions to the menu
    pub fn fill_menu(&self, actions: &mut Vec<Action>) {
        actions.push(Action::Back);
        actions.push(Action::NewDrawer);
        actions.push(Action::OpenDrawer);
        if let DrawerState::DrawerEdit(des) = &self.drawer_state {
            actions.push(Action::SaveDrawer);
            actions.push(Action::CloseDrawer);
            actions.push(Action::ToggleHiding);
            if des.drawer.content.settings.open_all_values {
                actions.push(Action::CloseAllValues);
            } else {
                actions.push(Action::OpenAllValues);
            }
        }
        actions.push(Action::Help);
        actions.push(Action::Quit);
    }

    /// Handle a key event
    pub fn on_key(&mut self, key: KeyEvent) -> Result<CmdResult, SafeClosetError> {
        use {
            DrawerFocus::*,
            DrawerState::*,
        };
        self.message = None;

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

        if let Some(action) = Action::for_key(key) {
            return self.on_action(action);
        }

        if let Some(menu_state) = self.menu.as_mut() {
            return menu_state.on_key(key)
                .map_or(Ok(CmdResult::Stay), |a| self.on_action(a));
        }

        if let Some(help_state) = &mut self.help {
            help_state.apply_key_event(key);
            return Ok(CmdResult::Stay);
        }

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
                        self.set_error(e.to_string());
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
                        self.set_error("This passphrase opens no drawer");
                    }
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
                    des.edit_entry_name_by_line(idx, EditionPos::Start); // as there's no filtering, idx==line
                } else if let NameSelected { line } = &des.focus {
                    let line = *line;
                    des.edit_entry_value_by_line(line, EditionPos::Start);
                } else if let NameEdit { line, .. } = &des.focus {
                    let line = *line;
                    des.close_input(false);
                    des.edit_entry_value_by_line(line, EditionPos::Start);
                } else if let ValueSelected { line } | ValueEdit { line, .. } = &des.focus {
                    let line = *line;
                    des.close_input(false);
                    if des.listed_entries_count() == line + 1 {
                        // last listed entry
                        if des.drawer.content.entries[line].is_empty() {
                            // if the current entry is empty, we don't create a new one
                            // but go back to the current (empty) entry name
                            des.edit_entry_name_by_line(line, EditionPos::Start);
                        } else {
                            // we create a new entry and start edit it
                            // but we must ensure there's no search which could filter it
                            des.search.clear();
                            des.drawer.content.entries.push(Entry::default());
                            des.edit_entry_name_by_line(
                                des.drawer.content.entries.len() - 1,
                                EditionPos::Start,
                            );
                        }
                    } else {
                        des.edit_entry_name_by_line(
                            line + 1,
                            EditionPos::Start,
                        );
                    }
                }
                des.update_search();
                return Ok(CmdResult::Stay);
            }
        }

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
                    des.edit_entry_name_by_line(line, EditionPos::Start);
                }
                if let ValueSelected { line } = &des.focus {
                    let line = *line;
                    des.edit_entry_value_by_line(line, EditionPos::Start);
                }
            }
            return Ok(CmdResult::Stay);
        }

        if as_letter(key) == Some('a') {
            if let DrawerEdit(des) = &mut self.drawer_state {
                if let NameSelected { line } = &des.focus {
                    let line = *line;
                    des.edit_entry_name_by_line(line, EditionPos::End);
                }
                if let ValueSelected { line } = &des.focus {
                    let line = *line;
                    des.edit_entry_value_by_line(line, EditionPos::End);
                }
            }
            return Ok(CmdResult::Stay);
        }

        if let DrawerEdit(des) = &mut self.drawer_state {
            if key == RIGHT {
                match &des.focus {
                    SearchEdit { previous_line } => {
                        // we're here because apply_event on the input returned false,
                        // which means the right arrow key was ignored because it was
                        // at the end of the input. We'll assume the user wants to
                        // select the value of the selected line
                        if let Some(line) = des.best_search_line() {
                            des.focus = ValueSelected { line };
                        } else if let Some(&line) = previous_line.as_ref() {
                            des.focus = ValueSelected { line };
                        }
                    }
                    NameSelected { line } => {
                        let line = *line;
                        des.focus = ValueSelected { line };
                    }
                    NoneSelected => {
                        des.focus = NameSelected { line: 0 };
                    }
                    _ => {}
                }
                return Ok(CmdResult::Stay);
            }
            if key == LEFT {
                match &des.focus {
                    NameSelected { .. } => {
                        des.focus = SearchEdit { previous_line: des.focus.line() };
                    }
                    ValueSelected { line } => {
                        let line = *line;
                        des.focus = NameSelected { line };
                    }
                    NoneSelected => {
                        des.focus = NameSelected { line: 0 };
                    }
                    _ => {}
                }
                return Ok(CmdResult::Stay);
            }
            if key == UP {
                des.move_line(Direction::Up);
                return Ok(CmdResult::Stay);
            }
            if key == DOWN {
                des.move_line(Direction::Down);
                return Ok(CmdResult::Stay);
            }
        }

        Ok(CmdResult::Stay)
    }
}
