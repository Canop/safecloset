use {
    super::*,
    crate::error::SafeClosetError,
    crossterm::{
        style::{Color, Color::*},
    },
    minimad::{Alignment, Composite},
    termimad::{ansi, gray, Area, CompoundStyle, MadSkin},
};

pub struct TitleView {
    area: Area,
    skin: MadSkin,
}

impl Default for TitleView {
    fn default() -> Self {
        Self {
            area: Area::uninitialized(),
            skin: make_skin(),
        }
    }
}

impl View for TitleView {
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
        let path = state.open_closet.path().to_string_lossy();
        let md = format!(" **SafeCloset** ` ` {} ` ` {} ", &path, state_info(state));
        let composite = Composite::from_inline(&md);
        self.go_to_line(w, 0)?;
        self.skin.write_composite_fill(w, composite, self.width(), Alignment::Unspecified)?;
        Ok(())
    }
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.paragraph.set_fgbg(AnsiValue(252), AnsiValue(239));
    skin.bold.set_fg(ansi(222));
    skin.italic = CompoundStyle::with_fg(AnsiValue(222));
    skin.inline_code.set_bg(gray(2));
    skin
}

fn state_info(state: &AppState) -> &'static str {
    let deep = state.open_closet.depth() > 0;
    match &state.drawer_state {
        DrawerState::NoneOpen => {
            if state.open_closet.just_created() && state.created_drawers == 0 {
                "new closet"
            } else {
                "no open drawer"
            }
        }
        DrawerState::DrawerCreation(_) => {
            if deep {
                "deep drawer creation"
            } else {
                "shallow drawer creation"
            }
        }
        DrawerState::DrawerOpening(_) => {
            if deep {
                "deep drawer opening"
            } else {
                "shallow drawer opening"
            }
        }
        DrawerState::DrawerEdit(des) => {
            if des.touched() {
                "*unsaved changes*"
            // } else if deep {
            //     "deep drawer"
            // } else {
            //     "shallow drawer"
            } else {
                ""
            }
        }
    }
}

