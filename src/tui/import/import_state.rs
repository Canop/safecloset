use {
    super::*,
    crate::{
        core::*,
        import::ImportSet,
        tui::menu::*,
    },
    crokey::{
        crossterm::event::{
            KeyEvent,
            MouseEvent,
        },
        key,
    },
    std::path::PathBuf,
};

pub enum Step {
    DecideOriginKind(Menu<OriginKind>),
    FileSelector(FileSelector),
    TypeDrawerPassword {
        open_closet: OpenCloset,
        dialog: PasswordDialog,
    },
    ConfirmImport {
        open_closet: OpenCloset,
        menu: Menu<ConfirmOrGoDeeper>,
        import_set: ImportSet,
    },
    InformEnd(InformMenu),
    Finished,
}
impl Default for Step {
    fn default() -> Self {
        let mut menu = Menu::new();
        menu.set_intro(
            "Importing adds content from another drawer.\n\
            This operation never removes content.",
        );
        menu.add_item(OriginKind::LocalFile, Some(key!(s)));
        menu.add_item(OriginKind::OtherFile, Some(key!(a)));
        Self::DecideOriginKind(menu)
    }
}

pub struct ImportState {
    pub dst_path: PathBuf,
    pub dst_drawer_state: DrawerState, // "borrowed" from the appstate
    pub step: Step,
    pub message: Option<&'static str>,
    pub hide_chars: bool,
}

impl ImportState {
    pub fn new(
        dst_path: PathBuf,
        dst_drawer_state: DrawerState,
    ) -> Self {
        Self {
            dst_path,
            dst_drawer_state,
            step: Step::default(),
            message: None,
            hide_chars: true,
        }
    }
    pub fn toggle_hide_chars(&mut self) {
        self.hide_chars = !self.hide_chars;
        if let Step::TypeDrawerPassword { dialog, .. } = &mut self.step {
            dialog.set_hide_chars(self.hide_chars);
        }
    }
    fn end<S: Into<String>>(
        &mut self,
        s: S,
    ) {
        self.step = Step::InformEnd(inform(s));
    }
    fn finish(&mut self) {
        self.step = Step::Finished;
    }
    // take the current step, putting Finished instead
    fn take_step(&mut self) -> Step {
        let mut step = Step::Finished;
        std::mem::swap(&mut step, &mut self.step);
        step
    }
    fn origin_kind_decided(
        &mut self,
        kind: OriginKind,
    ) {
        match kind {
            OriginKind::LocalFile => {
                self.on_file_selected(self.dst_path.clone());
            }
            OriginKind::OtherFile => {
                let file_selector = FileSelector::new(
                    "Enter the path of the file to import from.".to_string(),
                    FileType::Closet,
                );
                self.message = Some(file_selector.get_message());
                self.step = Step::FileSelector(file_selector);
            }
        }
    }
    fn on_file_selected(
        &mut self,
        path: PathBuf,
    ) {
        match OpenCloset::open(path) {
            Ok(open_closet) => {
                self.ask_password(open_closet);
            }
            Err(e) => {
                warn!("error opening file: {e}");
                self.end("An error prevented reopening the file");
            }
        }
    }
    fn ask_password(
        &mut self,
        open_closet: OpenCloset,
    ) {
        let mut dialog = PasswordDialog::new(
            PasswordDialogPurpose::OpenDrawer {
                depth: open_closet.depth(),
            },
            true,
        );
        dialog.set_hide_chars(self.hide_chars);
        self.step = Step::TypeDrawerPassword {
            open_closet,
            dialog,
        };
    }
    fn on_password(
        &mut self,
        dialog: PasswordDialog,
        mut open_closet: OpenCloset,
    ) {
        let password = dialog.get_password();
        if let Some(src_drawer) = open_closet.open_take_drawer(&password) {
            let import_set = ImportSet::new(src_drawer, &self.dst_drawer_state.drawer);
            let mut menu = Menu::new();
            if import_set.is_empty() {
                menu.set_intro(
                    "The selected drawer contains nothing which isn't already\
                    in the destination drawer."
                        .to_string(),
                );
            } else {
                menu.set_intro(import_set.confirm_string());
                menu.add_item(ConfirmOrGoDeeper::Confirm, None);
            }
            menu.add_item(ConfirmOrGoDeeper::GoDeeper, None);
            menu.add_item(ConfirmOrGoDeeper::Cancel, None);
            self.step = Step::ConfirmImport {
                open_closet,
                menu,
                import_set,
            };
        } else {
            info!("wrong pass");
            self.message = Some("Wrong passphrase");
            self.step = Step::TypeDrawerPassword {
                dialog,
                open_closet,
            };
        }
    }
    fn execute_import(
        &mut self,
        import_set: ImportSet,
    ) {
        let report = import_set.import_into(&mut self.dst_drawer_state.drawer);
        info!("import done");
        self.end(report);
    }
    pub fn is_finished(&self) -> bool {
        matches!(self.step, Step::Finished)
    }
    pub fn apply_key_event(
        &mut self,
        key: KeyEvent,
    ) -> bool {
        if key == key!(esc) {
            self.step = Step::Finished;
            return true;
        }
        let step = self.take_step();
        match step {
            Step::DecideOriginKind(mut menu) => match menu.state.on_key(key) {
                Some(kind) => {
                    self.origin_kind_decided(kind);
                    true
                }
                None => {
                    self.step = Step::DecideOriginKind(menu);
                    false
                }
            },
            Step::FileSelector(mut selector) => {
                let mut b = true;
                if key == key!(enter) {
                    if let Some(path) = selector.get_selected_file() {
                        self.on_file_selected(path.to_path_buf());
                    } else {
                        self.message = Some(selector.get_message());
                        self.step = Step::FileSelector(selector);
                    }
                } else {
                    b = selector.apply_key_event(key);
                    self.message = Some(selector.get_message());
                    self.step = Step::FileSelector(selector);
                }
                b
            }
            Step::TypeDrawerPassword {
                mut dialog,
                open_closet,
            } => {
                let mut b = true;
                if key == key!(enter) {
                    self.on_password(dialog, open_closet);
                } else {
                    b = dialog.apply_key_event(key);
                    self.step = Step::TypeDrawerPassword {
                        dialog,
                        open_closet,
                    };
                }
                b
            }
            Step::ConfirmImport {
                mut menu,
                open_closet,
                import_set,
            } => match menu.state.on_key(key) {
                Some(res) => {
                    match res {
                        ConfirmOrGoDeeper::Confirm => {
                            self.execute_import(import_set);
                        }
                        ConfirmOrGoDeeper::GoDeeper => {
                            self.ask_password(open_closet);
                        }
                        ConfirmOrGoDeeper::Cancel => {
                            info!("import canceled");
                            self.finish();
                        }
                    }
                    true
                }
                None => {
                    self.step = Step::ConfirmImport {
                        menu,
                        open_closet,
                        import_set,
                    };
                    false
                }
            },
            Step::InformEnd(mut menu) => {
                if menu.state.on_key(key).is_some() {
                    self.finish();
                } else {
                    self.step = Step::InformEnd(menu);
                }
                true
            }
            Step::Finished => true,
        }
    }
    /// handle a mouse event
    pub fn on_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        double_click: bool,
    ) {
        let step = self.take_step();
        match step {
            Step::DecideOriginKind(mut menu) => {
                if let Some(kind) = menu.state.on_mouse_event(mouse_event, double_click) {
                    self.origin_kind_decided(kind);
                } else {
                    self.step = Step::DecideOriginKind(menu);
                }
            }
            Step::FileSelector(mut selector) => {
                selector.on_mouse_event(mouse_event, double_click);
                self.step = Step::FileSelector(selector);
            }
            Step::TypeDrawerPassword {
                mut dialog,
                open_closet,
            } => {
                dialog.on_mouse_event(mouse_event, double_click);
                self.step = Step::TypeDrawerPassword {
                    dialog,
                    open_closet,
                };
            }
            Step::ConfirmImport {
                mut menu,
                open_closet,
                import_set,
            } => match menu.state.on_mouse_event(mouse_event, double_click) {
                Some(res) => match res {
                    ConfirmOrGoDeeper::Confirm => {
                        self.execute_import(import_set);
                    }
                    ConfirmOrGoDeeper::GoDeeper => {
                        self.ask_password(open_closet);
                    }
                    ConfirmOrGoDeeper::Cancel => {
                        info!("import canceled");
                        self.finish();
                    }
                },
                None => {
                    self.step = Step::ConfirmImport {
                        menu,
                        open_closet,
                        import_set,
                    };
                }
            },
            Step::InformEnd(mut menu) => {
                if menu
                    .state
                    .on_mouse_event(mouse_event, double_click)
                    .is_some()
                {
                    self.finish();
                } else {
                    self.step = Step::InformEnd(menu);
                }
            }
            Step::Finished => {}
        }
    }
    pub fn status(&self) -> &'static str {
        self.message.unwrap_or("Import wizard - WIP")
    }
}
