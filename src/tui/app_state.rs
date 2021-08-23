use {
    super::*,
    crate::{core::*, error::SafeClosetError},
    crossterm::{self, event::KeyEvent},
};

/// TUI Application state, containing a drawer state.
///
/// Needs a closet
pub struct AppState {
    pub closet: Closet,
    pub drawer_state: DrawerState,
    pub error: Option<String>,
}

impl AppState {
    pub fn new(closet: Closet) -> Self {
        Self {
            closet,
            drawer_state: DrawerState::NoneOpen,
            error: None,
        }
    }
    /// If there's an open drawer input (entry name or value), close it, keeping
    /// the input content if required.
    ///
    /// Return true if there was such input
    fn close_drawer_input(&mut self, discard: bool) -> bool {
        if let DrawerState::DrawerEdit(des) = &mut self.drawer_state {
            des.close_input(discard);
        }
        false
    }
    /// Save the content of the edited cell if any, then save the whole closet
    fn save(&mut self, reopen_if_open: bool) -> Result<(), SafeClosetError> {
        self.close_drawer_input(false);
        let drawer_state = std::mem::take(&mut self.drawer_state);
        if let DrawerState::DrawerEdit(des) = drawer_state {
            if reopen_if_open {
                self.drawer_state = DrawerState::DrawerEdit(des.close_and_reopen(&mut self.closet)?);
            } else {
                self.closet.close_drawer(des.drawer)?;
            }
        }
        self.closet.save()?;
        Ok(())
    }

    /// Handle a key event
    pub fn on_key(&mut self, key: KeyEvent) -> Result<CmdResult, SafeClosetError> {
        use DrawerState::*;
        self.error = None;

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

        // -- pending removal

        if let DrawerEdit(des) = &mut self.drawer_state {
            if let EntryState::PendingRemoval { idx } = &des.entry_state {
                let idx = *idx;
                // we either confirm (delete) or cancel removal
                if as_letter(key) == Some('y') {
                    info!("user requests entry removal");
                    des.drawer.entries.remove(idx);
                    des.entry_state = if idx > 0 {
                        EntryState::NameSelected { idx }
                    } else {
                        EntryState::NoneSelected
                    };
                } else {
                    info!("user cancels entry removal");
                    des.entry_state = EntryState::NameSelected { idx };
                }
            }
        }

        // -- toggle visibility of password or values

        if key == CONTROL_H {
            if let DrawerCreation(pis) | DrawerOpening(pis) = &mut self.drawer_state {
                pis.input.password_mode ^= true;
                return Ok(CmdResult::Stay);
            }
        }

        // --

        if key == ENTER {
            self.close_drawer_input(false); // if there's an entry input
            if let DrawerCreation(PasswordInputState { input }) = &mut self.drawer_state {
                let pwd = input.get_content();
                let open_drawer = self
                    .closet
                    .create_drawer(&pwd)
                    .map_err(|e| e.into())
                    .and_then(|_| {
                        self.closet.open_drawer(&pwd).ok_or_else(|| {
                            SafeClosetError::Internal(
                                "unexpected failure opening drawer".to_string(),
                            )
                        })
                    });
                match open_drawer {
                    Ok(open_drawer) => {
                        self.drawer_state = DrawerEdit(DrawerEditState::from(open_drawer));
                    }
                    Err(e) => {
                        warn!("error in drawer creation: {}", e);
                        self.error = Some(e.to_string());
                    }
                }
            } else if let DrawerOpening(PasswordInputState { input }) = &mut self.drawer_state {
                let pwd = input.get_content();
                let open_drawer = self.closet.open_drawer(&pwd);
                match open_drawer {
                    Some(open_drawer) => {
                        self.drawer_state = DrawerEdit(DrawerEditState::from(open_drawer));
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
                    des.entry_state = EntryState::NoneSelected;
                }
            }
            return Ok(CmdResult::Stay);
        }

        if let DrawerEdit(des) = &mut self.drawer_state {
            if key == TAB {
                if matches!(des.entry_state, EntryState::NoneSelected) {
                    let idx = des.drawer.empty_entry();
                    des.edit_entry_name(idx);
                } else if let EntryState::NameSelected { idx } = &des.entry_state {
                    let idx = *idx;
                    des.edit_entry_value(idx);
                } else if let EntryState::NameEdit { idx, .. } = &des.entry_state {
                    let idx = *idx;
                    des.close_input(false);
                    des.edit_entry_value(idx);
                } else if let EntryState::ValueSelected { idx } = &des.entry_state {
                    let idx = *idx;
                    if des.drawer.entries.len() == idx + 1 {
                        // last entry
                        if des.drawer.entries[idx].is_empty() {
                            // if the current entry is empty, we don't create a new one
                            // but go back to the current (empty) entry name
                            des.edit_entry_name(idx);
                        } else {
                            // we create a new entry and start edit it
                            des.drawer.entries.push(Entry::default());
                            des.edit_entry_name(idx + 1);
                        }
                    } else {
                        des.edit_entry_name(idx + 1);
                    }
                } else if let EntryState::ValueEdit { idx, .. } = &des.entry_state {
                    let idx = *idx;
                    des.close_input(false);
                    if des.drawer.entries.len() == idx + 1 {
                        // last entry
                        if des.drawer.entries[idx].is_empty() {
                            des.edit_entry_name(idx);
                        } else {
                            des.drawer.entries.push(Entry::default());
                            des.edit_entry_name(idx + 1);
                        }
                    } else {
                        des.edit_entry_name(idx + 1);
                    }
                }
                return Ok(CmdResult::Stay);
            }
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

        // --- input

        if let Some(input) = self.drawer_state.input() {
            input.apply_keycode_event(key.code);
            return Ok(CmdResult::Stay);
        }

        // --- navigation among entries

        if key == INSERT || as_letter(key) == Some('i') {
            if let DrawerEdit(des) = &mut self.drawer_state {
                if let EntryState::NameSelected { idx } = &des.entry_state {
                    let idx = *idx;
                    des.edit_entry_name(idx);
                }
                if let EntryState::ValueSelected { idx } = &des.entry_state {
                    let idx = *idx;
                    des.edit_entry_value(idx);
                }
            }
            return Ok(CmdResult::Stay);
        }

        if let DrawerEdit(des) = &mut self.drawer_state {
            if key == RIGHT {
                if let EntryState::NameSelected { idx } = &des.entry_state {
                    let idx = *idx;
                    des.entry_state = EntryState::ValueSelected { idx };
                }
                return Ok(CmdResult::Stay);
            }
            if key == LEFT {
                if let EntryState::ValueSelected { idx } = &des.entry_state {
                    let idx = *idx;
                    des.entry_state = EntryState::NameSelected { idx };
                }
                return Ok(CmdResult::Stay);
            }
            if key == UP {
                if let EntryState::NameSelected { idx } = &des.entry_state {
                    let idx = if *idx > 0 {
                        idx - 1
                    } else {
                        des.drawer.entries.len() - 1
                    };
                    des.entry_state = EntryState::NameSelected { idx };
                }
                if let EntryState::ValueSelected { idx } = &des.entry_state {
                    let idx = if *idx > 0 {
                        idx - 1
                    } else {
                        des.drawer.entries.len() - 1
                    };
                    des.entry_state = EntryState::ValueSelected { idx };
                }
                if matches!(des.entry_state, EntryState::NoneSelected) {
                    des.entry_state = EntryState::NameSelected { idx: 0 };
                }
                return Ok(CmdResult::Stay);
            }
            if key == DOWN {
                if let EntryState::NameSelected { idx } = &des.entry_state {
                    let idx = if *idx + 1 < des.drawer.entries.len() {
                        idx + 1
                    } else {
                        0
                    };
                    des.entry_state = EntryState::NameSelected { idx };
                }
                if let EntryState::ValueSelected { idx } = &des.entry_state {
                    let idx = if *idx + 1 < des.drawer.entries.len() {
                        idx + 1
                    } else {
                        0
                    };
                    des.entry_state = EntryState::ValueSelected { idx };
                }
                if matches!(des.entry_state, EntryState::NoneSelected) {
                    des.entry_state = EntryState::NameSelected { idx: 0 };
                }
                return Ok(CmdResult::Stay);
            }
        }

        // --- other simple char shortcuts

        if let Some(letter) = as_letter(key) {
            if matches!(self.drawer_state, NoneOpen) {
                match letter {
                    'n' => {
                        // new drawer
                        self.drawer_state = DrawerCreation(PasswordInputState::new(false));
                    }
                    'o' => {
                        // open drawer
                        self.drawer_state = DrawerOpening(PasswordInputState::new(true));
                    }
                    _ => {}
                }
                return Ok(CmdResult::Stay);
            }

            if let DrawerEdit(des) = &mut self.drawer_state {
                // if we're here, there's no input
                match (letter, des.entry_state.idx()) {
                    ('n', _) => {
                        // new entry
                        let idx = des.drawer.empty_entry();
                        des.edit_entry_name(idx);
                    }
                    ('d', Some(idx)) => {
                        // delete entry (with confirmation)
                        des.entry_state = EntryState::PendingRemoval { idx };
                    }
                    _ => {}
                }
                return Ok(CmdResult::Stay);
            }
        }

        Ok(CmdResult::Stay)
    }
}
