use {
    super::*,
    crate::error::SafeClosetError,
    crossterm::{
        style::{Color, Color::*},
    },
    minimad::{Alignment, Composite},
    termimad::{gray, Area, CompoundStyle, MadSkin},
};

/// The view giving hints or informing of an error, at
/// the bottom of the screen
pub struct StatusView {
    area: Area,
    hint_skin: MadSkin,
    error_skin: MadSkin,
    drawer_display_count: usize,
}

impl Default for StatusView {
    fn default() -> Self {
        Self {
            area: Area::uninitialized(),
            hint_skin: make_hint_skin(),
            error_skin: make_error_skin(),
            drawer_display_count: 0,
        }
    }
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
                hints.push("Hit *^q* to quit, *^s* to save, *^x* to save and quit");
            }
            if !des.drawer.content.entries.is_empty() {
                if matches!(des.focus, DrawerFocus::NameSelected{..}|DrawerFocus::ValueSelected{..}) {
                    hints.push("Hit *^q* to quit, *i* to edit the selected cell, *?* for help");
                }
                hints.push("Hit *^q* to quit, */* to search, *n* to create a new entry");
                hints.push("Hit *^q* to quit, *?* for help");
                hints.push("Hit *^q* to quit, */* to search, *^h* to toggle values visibility");
                hints.push("Hit *^q* to quit, */* to search, arrows to select a cell");
                hints.push("Hit *^q* to quit, *tab* to edit the next cell, *?* for help");
            }
            hints.push("Hit *^q* to quit, *?* for help");
        }
        let idx = (self.drawer_display_count / 3 ) % hints.len();
        self.drawer_display_count += 1;
        hints[idx]
    }
}

impl View for StatusView {
    fn set_area(&mut self, area: Area) {
        self.area = area;
    }
    fn get_area(&self) -> &Area {
        &self.area
    }
    fn bg(&self) -> Color {
        gray(4)
    }

    fn draw(&mut self, w: &mut W, state: &mut AppState) -> Result<(), SafeClosetError> {
        self.go_to_line(w, self.area.top)?;
        if let Some(Message { text, error }) = &state.message {
            let composite = Composite::from_inline(text);
            let skin = if *error {
                &self.error_skin
            } else {
                &self.hint_skin
            };
            skin.write_composite_fill(
                w,
                composite,
                self.width(),
                Alignment::Unspecified,
            )?;
        } else {
            let s = if state.help.is_some() {
                "Hit *^q* to quit, *esc* to close the help"
            } else if let DrawerState::DrawerEdit(des) = &state.drawer_state {
                self.rotate_drawer_hint(des)
            } else {
                "Hit *^q* to quit, *?* for help"
            };
            self.hint_skin.write_composite_fill(
                w,
                Composite::from_inline(s),
                self.width(),
                Alignment::Unspecified,
            )?;
        }
        Ok(())
    }
}

fn make_hint_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.paragraph.set_fgbg(AnsiValue(252), AnsiValue(239));
    skin.italic = CompoundStyle::with_fg(AnsiValue(222)); // 203 ?
    skin
}

fn make_error_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.paragraph.set_fgbg(AnsiValue(254), AnsiValue(160));
    skin
}
