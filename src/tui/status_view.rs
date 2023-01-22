use {
    super::*,
    crate::error::SafeClosetError,
    termimad::{
        minimad::{Alignment, Composite},
        Area,
    },
};

/// The view giving hints or informing of an error, at
/// the bottom of the screen
#[derive(Default)]
pub struct StatusView {
    area: Area,
    drawer_display_count: usize,
}

impl StatusView {

    /// return a hint for when a drawer is displayed
    fn rotate_drawer_hint(&mut self, ds: &DrawerState) -> &'static str {
        use DrawerFocus::*;
        let mut hints: Vec<&'static str> = Vec::new();
        match &ds.focus {
            NoneSelected if !ds.drawer.content.entries.is_empty() => {
                if ds.search.has_content() {
                    hints.push("Hit */* then *esc* to clear the search");
                }
                hints.push("Hit *^q* to quit, */* to search, *^h* to toggle values visibility");
                hints.push("Hit *^q* to save, */* to search, arrows to select a cell");
                hints.push("Hit *^q* to quit, *tab* or *n* to create a new entry");
                hints.push("Hit *^s* to save, *^q* to quit, *?* for help");
            }
            NoneSelected => {
                hints.push("Hit *^q* to quit, *tab* or *n* to create a new entry");
            }
            NameSelected { .. } | ValueSelected { .. } => {
                if ds.search.has_content() {
                    hints.push("Hit */* then *esc* to clear the search");
                }
                hints.push("Hit *^q* to quit, *i* to edit the selected cell, *?* for help");
                hints.push("Hit *^q* to quit, *i* to edit the selected cell, *esc* for menu");
                hints.push("Hit *^q* to quit, *i* or *a* to edit the selected cell, *esc* for menu");
                hints.push("Hit *^q* to quit, */* to search, *n* to create a new entry");
                hints.push("Hit *^q* to quit, */* to search, *^h* to toggle values visibility");
                hints.push("Hit *^q* to save, */* to search, arrows to select a cell");
                hints.push("Hit *^q* to quit, *tab* to edit the next cell");
                hints.push("Hit *^s* to save, *^q* to quit, *?* for help");
            }
            SearchEdit { .. } => {
                if ds.search.input.is_empty() {
                    hints.push("Hit *esc* to cancel search, or a few chars to filter entries");
                } else if ds.search.has_content() {
                    hints.push("Hit *esc* to cancel search, *enter* to keep the result");
                    hints.push("Hit *esc* to cancel search, arrows to keep the result and move selection");
                } else {
                    hints.push("Hit *esc* to cancel search");
                }
            }
            NameEdit { .. } | ValueEdit { .. } => {
                hints.push("Hit *esc* to cancel edition, *enter* to validate");
                hints.push("Hit *tab* to validate and go to next field");
            }
            PendingRemoval { .. } => {
                hints.push("Hit *y* to confirm entry removal (any other key cancels it)");
            }
        }
        if ds.touched() {
            hints.push("Hit *^s* to save, *^q* to quit, *esc* for menu");
        } else {
            hints.push("Hit *^q* to quit, *esc* for menu");
        }
        let idx = (self.drawer_display_count / 3 ) % hints.len();
        self.drawer_display_count += 1;
        hints[idx]
    }
}

impl View<AppState> for StatusView {

    fn set_available_area(&mut self, area: Area) {
        self.area = area;
    }

    fn draw(
        &mut self,
        w: &mut W,
        state: &mut AppState,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        w.go_to(self.area.left, self.area.top)?;
        let skin;
        let text;
        if let Some(task) = state.pending_tasks.get(0) {
            text = task.label();
            skin = &app_skin.status.task;
        } else if let Some(ref message) = &state.message {
            skin = if message.error {
                &app_skin.status.error
            } else {
                &app_skin.status.info
            };
            text = &message.text;
        } else {
            text = match &state.dialog {
                Dialog::None => {
                    if let Some(ds) = &state.drawer_state {
                        self.rotate_drawer_hint(ds)
                    } else {
                        "Hit *^q* to quit, *?* for help"
                    }
                }
                Dialog::Menu(_) => {
                    "Hit arrows to select an item, *enter* to validate, *esc* to close"
                }
                Dialog::Help(_) => {
                    "Hit *^q* to quit, *esc* to close the help"
                }
                Dialog::Password(_) => {
                    "Hit *esc* to cancel, *enter* to validate, *^q* to quit"
                }
                Dialog::CommentsEditor(_) => {
                    "Hit *esc* to cancel, *enter* to validate, *^q* to quit"
                }
            };
            skin = &app_skin.status.hint;
        }
        let composite = Composite::from_inline(text);
        skin.write_composite_fill(
            w,
            composite,
            self.area.width as usize,
            Alignment::Unspecified,
        )?;
        Ok(())
    }
}
