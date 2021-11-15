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
    /// the dialog (help, menu, etc.) displayed over the rest, if any
    pub dialog: Dialog,
    /// The message (error or info) displayed in the status bar
    pub message: Option<Message>,
    /// whether to hide unselected values
    pub hide_values: bool,
    /// number of drawers created during this session
    pub created_drawers: usize,
}

impl AppState {

    pub fn new(open_closet: OpenCloset, args: &Args) -> Self {
        let mut state = Self {
            open_closet,
            drawer_state: DrawerState::NoneOpen,
            dialog: Dialog::None,
            message: None,
            hide_values: args.hide,
            created_drawers: 0,
        };
        if args.open && !state.open_closet.just_created() {
            state.dialog = Dialog::Password(
                PasswordDialog::new(PasswordDialogPurpose::OpenDrawer, true)
            );
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
                let mut menu = Menu::default();
                menu.add_item(Action::ConfirmEntryRemoval);
                menu.add_item(Action::Back);
                menu.select(1);
                self.dialog = Dialog::Menu(menu);
            }
        }
    }
    fn cancel_entry_removal(&mut self) {
        if let DrawerState::DrawerEdit(des) = &mut self.drawer_state {
            if let DrawerFocus::PendingRemoval { line } = &des.focus {
                let line = *line;
                des.focus = DrawerFocus::NameSelected { line };
                self.dialog = Dialog::None;
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
                } else if self.dialog.is_some() {
                    self.dialog = Dialog::None;
                } else if self.close_drawer_input(true) {
                    debug!("closing drawer input");
                } else {
                    debug!("opening menu");
                    let mut menu = Menu::default();
                    self.fill_menu(&mut menu);
                    self.dialog = Dialog::Menu(menu);
                }
            }
            Action::NewDrawer => {
                self.dialog = Dialog::None;
                self.push_back_drawer()?;
                self.dialog = Dialog::Password(
                    PasswordDialog::new(
                        PasswordDialogPurpose::NewDrawer { depth: self.open_closet.depth() },
                        false,
                    )
                );
            }
            Action::OpenDrawer => {
                self.dialog = Dialog::None;
                self.push_back_drawer()?;
                self.dialog = Dialog::Password(
                    PasswordDialog::new(PasswordDialogPurpose::OpenDrawer, true)
                );
            }
            Action::SaveDrawer => {
                if self.drawer_state.is_edit() {
                    self.dialog = Dialog::None;
                    debug!("user requests save, keep state");
                    self.save(true)?;
                } else {
                    self.set_error("no open drawer");
                }
            }
            Action::CloseShallowDrawer | Action::CloseDeepDrawer => {
                self.dialog = Dialog::None;
                self.save(true)?;
                self.push_back_drawer()?;
                let _ = self.open_closet.close_deepest_drawer();
                self.drawer_state = match self.open_closet.take_deepest_open_drawer() {
                    Some(open_drawer) => DrawerState::edit(open_drawer),
                    None => DrawerState::NoneOpen,
                };
            }
            Action::Help => {
                self.dialog = Dialog::Help(Help::default());
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
                if let Dialog::Password(password_dialog) = &mut self.dialog {
                    password_dialog.toggle_hide_chars();
                    return Ok(CmdResult::Stay);
                }
                self.dialog = Dialog::None;
                if let DrawerEdit(des) = &mut self.drawer_state {
                    des.drawer.content.settings.hide_values ^= true;
                    return Ok(CmdResult::Stay);
                }
            }
            Action::OpenAllValues | Action::CloseAllValues=> {
                self.dialog = Dialog::None;
                if let DrawerEdit(des) = &mut self.drawer_state {
                    des.drawer.content.settings.open_all_values ^= true;
                    return Ok(CmdResult::Stay);
                }
            }
            Action::Copy => {
                self.dialog = Dialog::None;
                self.copy();
            }
            Action::Cut => {
                self.dialog = Dialog::None;
                self.cut();
            }
            Action::Paste => {
                self.dialog = Dialog::None;
                self.paste();
            }
            Action::ConfirmEntryRemoval => {
                self.dialog = Dialog::None;
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
                    self.dialog = Dialog::None;
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
                    if let Some(line) = des.focus.line() {
                        des.search.set_best_line(line);
                    }
                    let previous_idx = des.focus
                        .line()
                        .and_then(|line| des.listed_entry_idx(line));
                    des.focus = SearchEdit { previous_idx };
                }
            }
            Action::OpenPasswordChangeDialog => {
                debug!("opening pwd change dialog");
                self.dialog = Dialog::Password(PasswordDialog::new(
                    PasswordDialogPurpose::ChangeDrawerPassword,
                    false,
                ));
            }
        }
        Ok(CmdResult::Stay)
    }

    /// Add the relevant possible actions to the menu
    pub fn fill_menu(&self, menu: &mut Menu) {
        menu.add_item(Action::Back);
        menu.add_item(Action::NewDrawer);
        menu.add_item(Action::OpenDrawer);
        if let DrawerState::DrawerEdit(des) = &self.drawer_state {
            menu.add_item(Action::SaveDrawer);
            if self.open_closet.depth() > 1 {
                menu.add_item(Action::CloseDeepDrawer);
            } else {
                menu.add_item(Action::CloseShallowDrawer);
            }
            menu.add_item(Action::ToggleHiding);
            if des.drawer.content.settings.open_all_values {
                menu.add_item(Action::CloseAllValues);
            } else {
                menu.add_item(Action::OpenAllValues);
            }
            menu.add_item(Action::OpenPasswordChangeDialog);
        }
        menu.add_item(Action::Help);
        menu.add_item(Action::Quit);
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

        match &mut self.dialog {
            Dialog::Menu(menu) => {
                return menu.state.on_key(key)
                    .map_or(Ok(CmdResult::Stay), |a| self.on_action(a));
            }
            Dialog::Help(help) => {
                help.apply_key_event(key);
                return Ok(CmdResult::Stay);
            }
            Dialog::Password(password_dialog) => {
                if password_dialog.apply_key_event(key) {
                    return Ok(CmdResult::Stay);
                }
            }
            Dialog::None => {}
        }

        if key == ENTER {
            if let Dialog::Password(password_dialog) = &self.dialog {
                let password = password_dialog.get_password();
                match password_dialog.purpose() {
                    PasswordDialogPurpose::NewDrawer { .. } => {
                        let open_drawer = time!(self.open_closet.create_take_drawer(&password));
                        match open_drawer {
                            Ok(open_drawer) => {
                                self.drawer_state = DrawerState::edit(open_drawer);
                                self.created_drawers += 1;
                                self.dialog = Dialog::None;
                            }
                            Err(e) => {
                                self.set_error(e.to_string());
                            }
                        }
                    }
                    PasswordDialogPurpose::OpenDrawer => {
                        let open_drawer = self.open_closet.open_take_drawer(&password);
                        match open_drawer {
                            Some(mut open_drawer) => {
                                if self.hide_values {
                                    open_drawer.content.settings.hide_values = true;
                                }
                                self.drawer_state = DrawerState::edit(open_drawer);
                                self.dialog = Dialog::None;
                            }
                            None => {
                                self.set_error("This passphrase opens no drawer");
                            }
                        }
                    }
                    PasswordDialogPurpose::ChangeDrawerPassword => {
                        if let DrawerEdit(des) = &mut self.drawer_state {
                            match self.open_closet.change_password(&mut des.drawer, password) {
                                Ok(()) => {
                                    self.set_info(
                                        "Password changed. You should save then quit and try reopen."
                                    );
                                    self.dialog = Dialog::None;
                                }
                                Err(e) => {
                                    self.set_error(e.to_string());
                                }
                            }
                        }
                    }
                }
            } else {
                self.close_drawer_input(false); // if there's an entry input
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
                    SearchEdit { previous_idx } => {
                        let previous_line = previous_idx
                            .and_then(|idx| des.entry_line(idx));
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
                        let previous_idx = des.focus.line()
                            .and_then(|line| des.listed_entry_idx(line));
                        des.focus = SearchEdit { previous_idx };
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
