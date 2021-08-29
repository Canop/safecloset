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
}

impl Default for StatusView {
    fn default() -> Self {
        Self {
            area: Area::uninitialized(),
            hint_skin: make_hint_skin(),
            error_skin: make_error_skin(),
        }
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
        if let Some(error) = &state.error {
            let composite = Composite::from_inline(error);
            self.error_skin.write_composite_fill(
                w,
                composite,
                self.width(),
                Alignment::Unspecified,
            )?;
        } else {
            let s = if let DrawerState::DrawerEdit(des) = &state.drawer_state {
                if des.focus.is_pending_removal() {
                    "Hit *y* to confirm entry removal (any other key cancels it)"
                } else if des.touched() {
                    "Hit *^q* to quit, *^s* to save, *^x* to save and quit"
                } else if !des.drawer.entries.is_empty() {
                    "Hit *^q* to quit, *^h* to toggle unselected values visibility"
                } else {
                    "Hit *^q* to quit"
                }
            } else {
                "Hit *^q* to quit"
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
    skin.italic = CompoundStyle::with_fg(AnsiValue(44));
    skin
}

fn make_error_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.paragraph.set_fgbg(AnsiValue(254), AnsiValue(160));
    skin
}
