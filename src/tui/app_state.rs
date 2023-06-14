use {
    super::*,
    crate::{
        cli::Args,
        core::*,
        error::SafeClosetError,
    },
    crokey::*,
    crossterm::event::{
        KeyEvent,
        KeyModifiers,
        MouseButton,
        MouseEvent,
        MouseEventKind,
    },
    termimad::InputField,
};

/// TUI Application state
pub struct AppState {
    pub open_closet: OpenCloset,
    pub drawer_state: Option<DrawerState>,
    /// the dialog (help, menu, etc.) displayed over the rest, if any
    pub dialog: Dialog,
    /// The message (error or info) displayed in the status bar
    pub message: Option<Message>,
    /// whether to hide unselected values
    pub hide_values: bool,
    /// number of drawers created during this session
    pub created_drawers: usize,
    /// tasks in progress or waiting to be launched.
    /// The current or next one is at index 0
    pub pending_tasks: Vec<Task>,
}

impl AppState {
    pub fn new(
        open_closet: OpenCloset,
        args: &Args,
    ) -> Self {
        let dialog = if args.open && !open_closet.just_created() {
            Dialog::Password(PasswordDialog::new(
                PasswordDialogPurpose::OpenDrawer {
                    depth: open_closet.depth(),
                },
                true,
            ))
        } else {
            Dialog::None
        };
        Self {
            open_closet,
            drawer_state: None,
            dialog,
            message: None,
            hide_values: args.hide,
            created_drawers: 0,
            pending_tasks: Vec::new(),
        }
    }

    pub fn depth(&self) -> usize {
        self.open_closet.depth() + if self.drawer_state.is_some() { 1 } else { 0 }
    }

    fn set_error<S: Into<String>>(
        &mut self,
        error: S,
    ) {
        let text = error.into();
        warn!("error: {:?}", &text);
        self.message = Some(Message { text, error: true });
    }
    #[allow(dead_code)]
    fn set_info<S: Into<String>>(
        &mut self,
        info: S,
    ) {
        let text = info.into();
        debug!("info: {:?}", &text);
        self.message = Some(Message { text, error: false });
    }

    /// If there's an open drawer input (entry name or value), close it, keeping
    /// the input content if required.
    ///
    /// Return true if there was such input
    fn close_drawer_input(
        &mut self,
        discard: bool,
    ) -> bool {
        if let Some(ds) = &mut self.drawer_state {
            ds.close_input(discard)
        } else {
            false
        }
    }
    fn drawer_input(&mut self) -> Option<&mut InputField> {
        self.drawer_state
            .as_mut()
            .and_then(|ds| match &mut ds.focus {
                DrawerFocus::NameEdit { input, .. } => Some(input),
                DrawerFocus::ValueEdit { input, .. } => Some(input),
                DrawerFocus::SearchEdit { .. } => Some(&mut ds.search.input),
                _ => None,
            })
    }

    #[allow(dead_code)]
    fn is_on_entry_value(&self) -> bool {
        match self.drawer_state.as_ref() {
            Some(ds) => match &ds.focus {
                DrawerFocus::NameEdit { .. } => true,
                DrawerFocus::ValueEdit { .. } => true,
                _ => false,
            },
            _ => false,
        }
    }
    #[allow(dead_code)]
    fn has_input(&self) -> bool {
        match self.drawer_state.as_ref() {
            Some(ds) => match &ds.focus {
                DrawerFocus::NameEdit { .. } => true,
                DrawerFocus::ValueEdit { .. } => true,
                DrawerFocus::SearchEdit { .. } => true,
                _ => false,
            },
            _ => false,
        }
    }
    pub fn is_pending_removal(&self) -> bool {
        matches!(
            self.drawer_state,
            Some(DrawerState {
                focus: DrawerFocus::PendingRemoval { .. },
                ..
            }),
        )
    }

    /// Save the content of the edited cell if any, then save the whole closet
    fn save(
        &mut self,
        reopen_if_open: bool,
    ) -> Result<(), SafeClosetError> {
        time!(self.close_drawer_input(false));
        let drawer_state = std::mem::take(&mut self.drawer_state);
        if let Some(mut ds) = drawer_state {
            if reopen_if_open {
                self.drawer_state = Some(time!(ds.save_and_reopen(&mut self.open_closet)?));
            } else {
                ds.drawer.content.remove_empty_entries();
                time!(self.open_closet.push_back(ds.drawer)?);
                time!(self.open_closet.close_and_save())?;
            }
        } else {
            // saving only the closet
            time!(self.open_closet.close_and_save())?;
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
            if let Some(input) = self.drawer_input() {
                let s = input.copy_selection();
                if let Err(e) = terminal_clipboard::set_string(&s) {
                    self.set_error(e.to_string());
                } else if !s.is_empty() {
                    self.set_info("string copied to the clipboard, be cautious");
                }
            } else if let Some(ds) = &self.drawer_state {
                if let Some(cell) = ds.current_cell() {
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
            if let Some(input) = self.drawer_input() {
                let s = input.cut_selection();
                if let Err(e) = terminal_clipboard::set_string(&s) {
                    self.set_error(e.to_string());
                } else if !s.is_empty() {
                    self.set_info("string copied to the clipboard, be cautious");
                }
            } else {
                self.set_error("you can't copy from here");
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
                    if !self.is_on_entry_value() {
                        // we keep only the first line
                        pasted.truncate(pasted.lines().next().unwrap().len());
                    }
                    if let Some(input) = self.drawer_input() {
                        input.replace_selection(pasted);
                    } else if let Some(ds) = &mut self.drawer_state {
                        if let NameSelected { line } = &mut ds.focus {
                            let line = *line;
                            if ds.edit_entry_name_by_line(line, EditionPos::Start) {
                                if let Some(input) = self.drawer_input() {
                                    input.set_str(pasted);
                                    input.move_to_end();
                                    self.set_info("Hit *esc* to cancel pasting");
                                } else {
                                    warn!("unexpected lack of input");
                                }
                            }
                        } else if let ValueSelected { line } = &mut ds.focus {
                            let line = *line;
                            if ds.edit_entry_value_by_line(line, EditionPos::Start) {
                                if let Some(input) = self.drawer_input() {
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
    ) -> Result<(), SafeClosetError> {
        match &mut self.dialog {
            Dialog::Menu(menu) => {
                if let Some(action) = menu.state.on_mouse_event(mouse_event, double_click) {
                    self.on_action(action)?;
                }
                return Ok(());
            }
            Dialog::Help(help) => {
                help.on_mouse_event(mouse_event, double_click);
                return Ok(());
            }
            Dialog::Password(password_dialog) => {
                password_dialog.on_mouse_event(mouse_event, double_click);
                return Ok(());
            }
            Dialog::CommentsEditor(comments_editor) => {
                comments_editor.on_mouse_event(mouse_event, double_click);
                return Ok(());
            }
            Dialog::Import(import) => {
                import.on_mouse_event(mouse_event, double_click);
                if import.is_finished() {
                    self.dialog = Dialog::None;
                }
            }
            Dialog::None => {}
        }

        if let Some(input) = self.drawer_input() {
            if input.apply_mouse_event(mouse_event, double_click) {
                return Ok(());
            } else if let Some(ds) = &mut self.drawer_state {
                // unfocusing the input, validating it
                ds.focus = DrawerFocus::NoneSelected;
            }
        }
        if let Some(ds) = &mut self.drawer_state {
            let MouseEvent {
                kind,
                row,
                column,
                modifiers,
            } = mouse_event;
            match kind {
                MouseEventKind::Up(MouseButton::Left) => {
                    if modifiers == KeyModifiers::NONE {
                        // The case of an input being focused is handled before
                        // so we know it's not the case
                        match ds.clicked(column, row as usize) {
                            Clicked::Search => {
                                self.on_action(Action::Search)?;
                            }
                            Clicked::Name(clicked_line) => {
                                if ds.focus.is_name_selected(clicked_line) {
                                    ds.edit_entry_name_by_line(clicked_line, EditionPos::Start);
                                } else {
                                    ds.focus = DrawerFocus::NameSelected { line: clicked_line };
                                }
                            }
                            Clicked::Value(clicked_line) => {
                                if ds.focus.is_value_selected(clicked_line) {
                                    ds.edit_entry_value_by_line(clicked_line, EditionPos::Start);
                                } else {
                                    ds.focus = DrawerFocus::ValueSelected { line: clicked_line };
                                }
                            }
                            Clicked::Nothing => {}
                        }
                    }
                }
                MouseEventKind::ScrollUp => {
                    ds.apply_scroll_command(ScrollCommand::Lines(-1));
                }
                MouseEventKind::ScrollDown => {
                    ds.apply_scroll_command(ScrollCommand::Lines(1));
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
        if let Some(DrawerState { drawer, .. }) = drawer_state {
            self.open_closet.push_back(drawer)?;
        }
        Ok(())
    }

    /// delete entry (with confirmation)
    fn propose_entry_removal(&mut self) {
        if let Some(ds) = &mut self.drawer_state {
            if let Some(line) = ds.focus.line() {
                ds.focus = DrawerFocus::PendingRemoval { line };
                let mut menu = ActionMenu::new();
                menu.add_action(Action::ConfirmEntryRemoval);
                menu.add_action(Action::Back);
                menu.state.select(1);
                self.dialog = Dialog::Menu(menu);
            }
        }
    }
    fn cancel_entry_removal(&mut self) {
        if let Some(ds) = &mut self.drawer_state {
            if let DrawerFocus::PendingRemoval { line } = &ds.focus {
                let line = *line;
                ds.focus = DrawerFocus::NameSelected { line };
                self.dialog = Dialog::None;
            }
        }
    }

    pub fn has_pending_task(&self) -> bool {
        !self.pending_tasks.is_empty()
    }

    pub fn queue_task(
        &mut self,
        task: Task,
    ) {
        self.pending_tasks.push(task);
    }

    fn shift_pending_task(&mut self) -> Option<Task> {
        if self.pending_tasks.is_empty() {
            None
        } else {
            Some(self.pending_tasks.remove(0))
        }
    }

    /// execute one of the potentially long tasks
    /// (the status is updated during execution)
    pub fn run_pending_task(&mut self) -> Result<CmdResult, SafeClosetError> {
        match self.shift_pending_task() {
            Some(Task::Save) => {
                self.save(true)?;
            }
            Some(Task::CreateDrawer(password)) => {
                self.push_back_drawer()?;
                let open_drawer = time!(self.open_closet.create_take_drawer(&password));
                match open_drawer {
                    Ok(open_drawer) => {
                        self.drawer_state = Some(open_drawer.into());
                        self.created_drawers += 1;
                        self.dialog = Dialog::None;
                    }
                    Err(e) => {
                        self.set_error(e.to_string());
                    }
                }
            }
            Some(Task::OpenDrawer(password)) => {
                self.push_back_drawer()?;
                let open_drawer = self.open_closet.open_take_drawer(&password);
                match open_drawer {
                    Some(mut open_drawer) => {
                        if self.hide_values {
                            open_drawer.content.settings.hide_values = true;
                        }
                        self.drawer_state = Some(open_drawer.into());
                        self.dialog = Dialog::None;
                    }
                    None => {
                        self.drawer_state = self
                            .open_closet
                            .take_deepest_open_drawer()
                            .map(|open_drawer| open_drawer.into());
                        self.set_error("This passphrase opens no drawer");
                    }
                }
            }
            Some(Task::CloseDrawer) => {
                self.push_back_drawer()?;
                let _ = self.open_closet.close_deepest_drawer();
                self.drawer_state = self
                    .open_closet
                    .take_deepest_open_drawer()
                    .map(|open_drawer| open_drawer.into());
            }
            Some(Task::ChangePassword(password)) => {
                if let Some(ds) = &mut self.drawer_state {
                    match self.open_closet.change_password(&mut ds.drawer, password) {
                        Ok(()) => {
                            self.set_info(
                                "Password changed. You should save then quit and try reopen.",
                            );
                            self.dialog = Dialog::None;
                        }
                        Err(e) => {
                            self.set_error(e.to_string());
                        }
                    }
                }
            }
            None => {
                warn!("unexpected lack of task");
            }
        }
        Ok(CmdResult::Stay)
    }

    pub fn on_action(
        &mut self,
        action: Action,
    ) -> Result<CmdResult, SafeClosetError> {
        use DrawerFocus::*;
        debug!("executing action {:?}", action);
        match action {
            Action::Back => {
                if self.is_pending_removal() {
                    self.cancel_entry_removal();
                } else if self.dialog.is_some() {
                    self.dialog = Dialog::None;
                } else if self.close_drawer_input(true) {
                    debug!("closing drawer input");
                } else {
                    debug!("opening menu");
                    let mut menu = ActionMenu::new();
                    self.fill_menu(&mut menu);
                    self.dialog = Dialog::Menu(menu);
                }
            }
            Action::NewDrawer => {
                self.dialog = Dialog::Password(PasswordDialog::new(
                    PasswordDialogPurpose::NewDrawer {
                        depth: self.depth(),
                    },
                    false,
                ));
            }
            Action::OpenDrawer => {
                self.dialog = Dialog::Password(PasswordDialog::new(
                    PasswordDialogPurpose::OpenDrawer {
                        depth: self.depth(),
                    },
                    true,
                ));
            }
            Action::Import => {
                if let Some(ds) = self.drawer_state.take() {
                    self.dialog = Dialog::Import(Import::new(self.open_closet.path().into(), ds));
                } else {
                    warn!("What ? How was this option chosen ?");
                }
            }
            Action::EditClosetComments => {
                self.dialog = Dialog::CommentsEditor(CommentsEditor::new(
                    &self.open_closet.root_closet().comments,
                ));
            }
            Action::SaveDrawer => {
                if self.drawer_state.is_some() {
                    self.dialog = Dialog::None;
                    debug!("user requests save, keep state");
                    self.queue_task(Task::Save);
                } else {
                    self.set_error("no open drawer");
                }
            }
            Action::CloseShallowDrawer | Action::CloseDeepDrawer => {
                self.dialog = Dialog::None;
                self.queue_task(Task::Save);
                self.queue_task(Task::CloseDrawer);
            }
            Action::Help => {
                self.dialog = Dialog::Help(Help::default());
            }
            Action::Quit => {
                debug!("user requests quit");
                return Ok(CmdResult::Quit);
            }
            Action::MoveLineUp => {
                if let Some(ds) = &mut self.drawer_state {
                    let entries = &mut ds.drawer.content.entries;
                    let len = entries.len();
                    match &mut ds.focus {
                        NameSelected { line } => {
                            let new_line = (*line + len - 1) % len;
                            entries.swap(*line, new_line);
                            ds.focus = NameSelected { line: new_line };
                        }
                        ValueSelected { line } => {
                            let new_line = (*line + len - 1) % len;
                            entries.swap(*line, new_line);
                            ds.focus = ValueSelected { line: new_line };
                        }
                        ValueEdit { input, .. } => {
                            input.move_current_line_up();
                        }
                        _ => {}
                    }
                    ds.update_search();
                }
            }
            Action::MoveLineDown => {
                if let Some(ds) = &mut self.drawer_state {
                    let entries = &mut ds.drawer.content.entries;
                    let len = entries.len();
                    match &mut ds.focus {
                        NameSelected { line } => {
                            let new_line = (*line + 1) % len;
                            entries.swap(*line, new_line);
                            ds.focus = NameSelected { line: new_line };
                        }
                        ValueSelected { line } => {
                            let new_line = (*line + 1) % len;
                            entries.swap(*line, new_line);
                            ds.focus = ValueSelected { line: new_line };
                        }
                        ValueEdit { input, .. } => {
                            input.move_current_line_down();
                        }
                        _ => {}
                    }
                    ds.update_search();
                }
            }
            Action::GroupMatchingEntries => {
                self.dialog = Dialog::None;
                if let Some(ds) = &mut self.drawer_state {
                    ds.group_matching_entries();
                }
            }
            Action::ToggleHiding => {
                // toggle visibility of password or values
                if let Dialog::Password(password_dialog) = &mut self.dialog {
                    password_dialog.toggle_hide_chars();
                    return Ok(CmdResult::Stay);
                }
                if let Dialog::Import(import) = &mut self.dialog {
                    import.toggle_hide_chars();
                    return Ok(CmdResult::Stay);
                }
                self.dialog = Dialog::None;
                if let Some(ds) = &mut self.drawer_state {
                    ds.drawer.content.settings.hide_values ^= true;
                }
            }
            Action::ToggleMarkdown => {
                self.dialog = Dialog::None;
                if let Some(ds) = &mut self.drawer_state {
                    ds.drawer.content.settings.values_as_markdown ^= true;
                }
            }
            Action::OpenAllValues | Action::CloseAllValues => {
                self.dialog = Dialog::None;
                if let Some(ds) = &mut self.drawer_state {
                    ds.drawer.content.settings.open_all_values ^= true;
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
                if let Some(ds) = &mut self.drawer_state {
                    if let PendingRemoval { line } = &ds.focus {
                        let line = *line;
                        if let Some(idx) = ds.listed_entry_idx(line) {
                            // we either confirm (delete) or cancel removal
                            ds.drawer.content.entries.remove(idx);
                            ds.focus = if line > 0 {
                                NameSelected { line: line - 1 }
                            } else {
                                NoneSelected
                            };
                            ds.update_search();
                        }
                    }
                }
            }
            Action::NewEntry => {
                if let Some(ds) = &mut self.drawer_state {
                    self.dialog = Dialog::None;
                    ds.search.clear();
                    let idx = ds.drawer.content.empty_entry();
                    ds.edit_entry_name_by_line(idx, EditionPos::Start);
                }
            }
            Action::NewEntryAfterCurrent => {
                if let Some(ds) = &mut self.drawer_state {
                    self.dialog = Dialog::None;
                    ds.search.clear();
                    let idx = ds.focus.line().and_then(|line| ds.listed_entry_idx(line));
                    let idx = ds.drawer.content.insert_after(idx);
                    ds.edit_entry_name_by_line(idx, EditionPos::Start);
                }
            }
            Action::RemoveLine => {
                self.propose_entry_removal();
            }
            Action::Search => {
                if let Some(ds) = &mut self.drawer_state {
                    if let Some(line) = ds.focus.line() {
                        ds.search.set_best_line(line);
                    }
                    let previous_idx = ds.focus.line().and_then(|line| ds.listed_entry_idx(line));
                    ds.focus = SearchEdit { previous_idx };
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
    pub fn fill_menu(
        &self,
        menu: &mut ActionMenu,
    ) {
        menu.add_action(Action::Back);
        menu.add_action(Action::NewDrawer);
        menu.add_action(Action::OpenDrawer);
        if let Some(ds) = &self.drawer_state {
            menu.add_action(Action::SaveDrawer);
            if self.depth() > 1 {
                menu.add_action(Action::CloseDeepDrawer);
            } else {
                menu.add_action(Action::CloseShallowDrawer);
            }
            menu.add_action(Action::ToggleHiding);
            menu.add_action(Action::ToggleMarkdown);
            if ds.drawer.content.settings.open_all_values {
                menu.add_action(Action::CloseAllValues);
            } else {
                menu.add_action(Action::OpenAllValues);
            }
            if ds.match_count() > 1 {
                menu.add_action(Action::GroupMatchingEntries);
            }
            menu.add_action(Action::OpenPasswordChangeDialog);
            menu.add_action(Action::Import);
        } else {
            menu.add_action(Action::EditClosetComments);
        }
        menu.add_action(Action::Help);
        menu.add_action(Action::Quit);
    }

    /// Handle a key event
    pub fn on_key(
        &mut self,
        key: KeyEvent,
    ) -> Result<CmdResult, SafeClosetError> {
        use DrawerFocus::*;
        self.message = None;

        if let Some(input) = self.drawer_input() {
            if input.apply_key_event(key) {
                if let Some(ds) = &mut self.drawer_state {
                    if ds.focus.is_search() {
                        ds.search.update(&ds.drawer);
                    }
                }
                return Ok(CmdResult::Stay);
            }
        }

        match &mut self.dialog {
            Dialog::Menu(menu) => {
                return menu
                    .state
                    .on_key(key)
                    .map_or(Ok(CmdResult::Stay), |a| self.on_action(a));
            }
            Dialog::Help(help) => {
                if help.apply_key_event(key) {
                    return Ok(CmdResult::Stay);
                }
            }
            Dialog::Password(password_dialog) => {
                if password_dialog.apply_key_event(key) {
                    return Ok(CmdResult::Stay);
                }
            }
            Dialog::CommentsEditor(comments_editor) => {
                if comments_editor.apply_key_event(key) {
                    return Ok(CmdResult::Stay);
                }
            }
            Dialog::Import(import) => {
                if import.on_key(key) {
                    if import.is_finished() {
                        let mut temp = Dialog::None;
                        std::mem::swap(&mut temp, &mut self.dialog);
                        if let Dialog::Import(import) = temp {
                            self.drawer_state = Some(import.take_back_drawer());
                        }
                    }
                    return Ok(CmdResult::Stay);
                }
            }
            Dialog::None => {}
        }

        if let Some(action) = Action::for_key(key) {
            return self.on_action(action);
        }

        if key == key!(enter) {
            match &mut self.dialog {
                Dialog::Password(password_dialog) => {
                    let password = password_dialog.get_password();
                    match password_dialog.purpose() {
                        PasswordDialogPurpose::NewDrawer { .. } => {
                            self.queue_task(Task::CreateDrawer(password));
                        }
                        PasswordDialogPurpose::OpenDrawer { .. } => {
                            self.queue_task(Task::OpenDrawer(password));
                        }
                        PasswordDialogPurpose::ChangeDrawerPassword => {
                            self.queue_task(Task::ChangePassword(password));
                        }
                    }
                }
                Dialog::Help(_) => {}
                Dialog::Menu(_) => {} // managed in the menu
                Dialog::CommentsEditor(ce) => {
                    self.open_closet.root_closet().comments = ce.get_comments();
                    self.dialog = Dialog::None;
                    self.queue_task(Task::Save);
                }
                Dialog::None => {
                    self.close_drawer_input(false); // if there's an entry input
                }
                Dialog::Import(_) => {} // managed in the dialog
            }
            return Ok(CmdResult::Stay);
        }

        if key == key!(tab) && self.dialog.is_none() {
            if let Some(ds) = &mut self.drawer_state {
                if matches!(ds.focus, NoneSelected) {
                    // we remove any search
                    ds.search.clear();
                    let idx = ds.drawer.content.empty_entry();
                    ds.edit_entry_name_by_line(idx, EditionPos::Start); // as there's no filtering, idx==line
                } else if let NameSelected { line } = &ds.focus {
                    let line = *line;
                    ds.edit_entry_value_by_line(line, EditionPos::Start);
                } else if let NameEdit { line, .. } = &ds.focus {
                    let line = *line;
                    ds.close_input(false);
                    ds.edit_entry_value_by_line(line, EditionPos::Start);
                } else if let ValueSelected { line } | ValueEdit { line, .. } = &ds.focus {
                    let line = *line;
                    ds.close_input(false);
                    if ds.listed_entries_count() == line + 1 {
                        // last listed entry
                        if ds.drawer.content.entries[line].is_empty() {
                            // if the current entry is empty, we don't create a new one
                            // but go back to the current (empty) entry name
                            ds.edit_entry_name_by_line(line, EditionPos::Start);
                        } else {
                            // we create a new entry and start edit it
                            // but we must ensure there's no search which could filter it
                            ds.search.clear();
                            ds.drawer.content.entries.push(Entry::default());
                            ds.edit_entry_name_by_line(
                                ds.drawer.content.entries.len() - 1,
                                EditionPos::Start,
                            );
                        }
                    } else {
                        ds.edit_entry_name_by_line(line + 1, EditionPos::Start);
                    }
                }
                ds.update_search();
                return Ok(CmdResult::Stay);
            }
        }

        if let Some(ds) = &mut self.drawer_state {
            if key == key!(home) {
                ds.apply_scroll_command(ScrollCommand::Top);
                return Ok(CmdResult::Stay);
            }
            if key == key!(end) {
                ds.apply_scroll_command(ScrollCommand::Bottom);
                return Ok(CmdResult::Stay);
            }
            if key == key!(pageup) {
                ds.apply_scroll_command(ScrollCommand::Pages(-1));
                return Ok(CmdResult::Stay);
            }
            if key == key!(pagedown) {
                ds.apply_scroll_command(ScrollCommand::Pages(1));
                return Ok(CmdResult::Stay);
            }
        }

        if key == key!(insert) || as_letter(key) == Some('i') {
            if let Some(ds) = &mut self.drawer_state {
                if let NameSelected { line } = &ds.focus {
                    let line = *line;
                    ds.edit_entry_name_by_line(line, EditionPos::Start);
                }
                if let ValueSelected { line } = &ds.focus {
                    let line = *line;
                    ds.edit_entry_value_by_line(line, EditionPos::Start);
                }
            }
            return Ok(CmdResult::Stay);
        }

        if as_letter(key) == Some('a') {
            if let Some(ds) = &mut self.drawer_state {
                if let NameSelected { line } = &ds.focus {
                    let line = *line;
                    ds.edit_entry_name_by_line(line, EditionPos::End);
                }
                if let ValueSelected { line } = &ds.focus {
                    let line = *line;
                    ds.edit_entry_value_by_line(line, EditionPos::End);
                }
            }
            return Ok(CmdResult::Stay);
        }

        if let Some(ds) = &mut self.drawer_state {
            if key == key!(right) {
                match &ds.focus {
                    SearchEdit { previous_idx } => {
                        let previous_line = previous_idx.and_then(|idx| ds.entry_line(idx));
                        // we're here because apply_event on the input returned false,
                        // which means the right arrow key was ignored because it was
                        // at the end of the input. We'll assume the user wants to
                        // select the value of the selected line
                        if let Some(line) = ds.best_search_line() {
                            ds.focus = ValueSelected { line };
                        } else if let Some(&line) = previous_line.as_ref() {
                            ds.focus = ValueSelected { line };
                        }
                    }
                    NameSelected { line } => {
                        let line = *line;
                        ds.focus = ValueSelected { line };
                    }
                    NoneSelected => {
                        ds.focus = NameSelected { line: 0 };
                    }
                    _ => {}
                }
                return Ok(CmdResult::Stay);
            }
            if key == key!(left) {
                match &ds.focus {
                    NameSelected { .. } => {
                        let previous_idx =
                            ds.focus.line().and_then(|line| ds.listed_entry_idx(line));
                        ds.focus = SearchEdit { previous_idx };
                    }
                    ValueSelected { line } => {
                        let line = *line;
                        ds.focus = NameSelected { line };
                    }
                    NoneSelected => {
                        ds.focus = NameSelected { line: 0 };
                    }
                    _ => {}
                }
                return Ok(CmdResult::Stay);
            }
            if key == key!(up) {
                ds.move_line(Direction::Up);
                return Ok(CmdResult::Stay);
            }
            if key == key!(down) {
                ds.move_line(Direction::Down);
                return Ok(CmdResult::Stay);
            }
        }

        Ok(CmdResult::Stay)
    }
}
