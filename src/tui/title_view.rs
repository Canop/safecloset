use {
    super::*,
    crate::error::SafeClosetError,
    crossterm::{
        style::{Color, Color::*},
    },
    minimad::{mad_inline, Alignment},
    termimad::{ansi, gray, Area, MadSkin},
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
        let path = state.closet.path().to_string_lossy();
        let composite = mad_inline!(" **SafeCloset** ` ` $0 ` ` ", &path,);
        self.go_to_line(w, 0)?;
        self.skin
            .write_composite_fill(w, composite, self.width(), Alignment::Unspecified)?;
        Ok(())
    }
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.paragraph.set_fgbg(AnsiValue(252), AnsiValue(239));
    skin.bold.set_fg(ansi(222));
    skin.inline_code.set_bg(gray(2));
    skin
}
