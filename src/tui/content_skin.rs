use {
    crossterm::{
        style::{
            Color,
            Color::*,
        },
    },
    termimad::{gray, CompoundStyle, MadSkin},
};

pub struct ContentSkin {

    /// view background
    pub bg: Color,

    pub name_fg: Color,

    pub sel_bg: Color,

    /// style of pattern matching chars
    pub char_match: CompoundStyle,

    /// style of pattern matching chars whith selection background
    pub sel_char_match: CompoundStyle,

    /// skin for all the markdown parts, when not applying a selection bagkground
    pub md: MadSkin,

    /// skin for the markdown parts with a selection background
    pub sel_md: MadSkin,
}

impl ContentSkin {
    pub fn new() -> Self {
        let bg = gray(2);
        let mut md = MadSkin::default();
        md.set_bg(bg);
        let sel_bg = gray(6);
        let mut sel_md = md.clone();
        sel_md.set_bg(sel_bg);
        //let char_match_fg = AnsiValue(203);
        let char_match_fg = AnsiValue(41);
        let char_match = CompoundStyle::with_fgbg(char_match_fg, bg);
        let sel_char_match = CompoundStyle::with_fgbg(char_match_fg, sel_bg);
        Self {
            bg,
            name_fg: AnsiValue(230),
            sel_bg,
            char_match,
            sel_char_match,
            md,
            sel_md,
        }
    }
    pub fn normal_style(&self) -> &CompoundStyle {
        &self.md.paragraph.compound_style
    }
    pub fn match_style(&self, selected: bool) -> &CompoundStyle {
        if selected {
            &self.sel_char_match
        } else {
            &self.char_match
        }
    }
    pub fn txt_style(&self, selected: bool) -> &CompoundStyle {
        if selected {
            &self.sel_md.paragraph.compound_style
        } else {
            &self.md.paragraph.compound_style
        }
    }
    pub fn tbl_style(&self, selected: bool) -> &CompoundStyle {
        if selected {
            &self.sel_md.table.compound_style
        } else {
            &self.md.table.compound_style
        }
    }
}
