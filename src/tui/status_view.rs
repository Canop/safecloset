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

    /// return a hint for the most normal case: no search, some entries, no
    /// special state
    fn rotate_drawer_hint(&mut self, des: &DrawerEditState) -> &'static str {
        let mut hints: Vec<&'static str> = Vec::new();
        if des.focus.is_pending_removal() {
            hints.push("Hit *y* to confirm entry removal (any other key cancels it)");
        } else if des.focus.is_search() {
            hints.push("Hit *esc* to cancel search, *enter* to keep the result");
            hints.push("Hit *esc* to cancel search, arrows to keep the result and move selection");
        } else {
            if des.search.has_content() {
                hints.push("Hit */* then *esc* to clear the search");
            }
            if des.touched() {
                hints.push("Hit *^s* to save, *^q* to quit, *esc* for menu");
            }
            if !des.drawer.content.entries.is_empty() {
                if matches!(des.focus, DrawerFocus::NameSelected{..}|DrawerFocus::ValueSelected{..}) {
                    hints.push("Hit *^q* to quit, *i* to edit the selected cell, *?* for help");
                    hints.push("Hit *^q* to quit, *i* to edit the selected cell, *esc* for menu");
                }
                hints.push("Hit *^q* to quit, */* to search, *n* to create a new entry");
                hints.push("Hit *^q* to quit, */* to search, *^h* to toggle values visibility");
                hints.push("Hit *^q* to save, */* to search, arrows to select a cell");
                hints.push("Hit *^q* to quit, *tab* to edit the next cell");
                if !des.has_input() {
                    hints.push("Hit *^s* to save, *^q* to quit, *?* for help");
                    hints.push("Hit *^s* to save, *^q* to quit, *esc* for the menu");
                }
            }
            if !des.has_input() {
                hints.push("Hit *^q* to quit, *esc* for the menu");
                hints.push("Hit *^q* to quit, *?* for help");
            }
        }
        let idx = (self.drawer_display_count / 3 ) % hints.len();
        self.drawer_display_count += 1;
        hints[idx]
    }
}

impl View for StatusView {
    type State = AppState;

    fn set_available_area(&mut self, area: Area) {
        self.area = area;
    }
    fn get_area(&self) -> &Area {
        &self.area
    }

    fn draw(
        &mut self,
        w: &mut W,
        state: &mut AppState,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        self.go_to_line(w, self.area.top)?;
        let skin;
        let text;
        if state.drawer_state.is_pending_removal() {
            text = "Hit *y* to confirm entry removal or *esc* to cancel it";
            skin = &app_skin.status.info;
        } else if let Some(ref message) = &state.message {
            skin = if message.error {
                &app_skin.status.error
            } else {
                &app_skin.status.info
            };
            text = &message.text;
        } else {
            text = if state.dialog.is_help() {
                "Hit *^q* to quit, *esc* to close the help"
            } else if let DrawerState::DrawerEdit(des) = &state.drawer_state {
                self.rotate_drawer_hint(des)
            } else if matches!(state.drawer_state, DrawerState::NoneOpen) {
                "Hit *^q* to quit, *?* for help"
            } else {
                "Hit *^q* to quit"
            };
            skin = &app_skin.status.hint;
        }
        let composite = Composite::from_inline(text);
        skin.write_composite_fill(
            w,
            composite,
            self.get_area().width as usize,
            Alignment::Unspecified,
        )?;
        Ok(())
    }
}
