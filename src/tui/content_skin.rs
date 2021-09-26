use {
    crossterm::{
        style::{
            Color,
            Color::*,
        },
    },
    termimad::{
        ansi, gray,
        CompoundStyle,
        InputField,
        MadSkin,
        ScrollBarStyle,
    },
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

    /// style for the global scrollbar when not active because of a focused input
    pub unsel_scrollbar: ScrollBarStyle,
}

impl ContentSkin {
    pub fn new() -> Self {
        let bg = gray(2);
        let mut md = MadSkin::default();
        md.set_fg(ansi(230));
        md.italic = CompoundStyle::with_fg(AnsiValue(222));
        md.set_bg(bg);
        let sel_bg = gray(5);
        let mut sel_md = md.clone();
        sel_md.set_bg(sel_bg);
        sel_md.scrollbar.thumb.set_fg(gray(10));
        sel_md.scrollbar.track.set_fg(gray(5));
        let char_match_fg = AnsiValue(41);
        let char_match = CompoundStyle::with_fgbg(char_match_fg, bg);
        let sel_char_match = CompoundStyle::with_fgbg(char_match_fg, sel_bg);
        let mut unsel_scrollbar = sel_md.scrollbar.clone();
        unsel_scrollbar.thumb.set_fg(gray(10));
        unsel_scrollbar.track.set_fg(gray(5));
        Self {
            bg,
            name_fg: AnsiValue(230),
            sel_bg,
            char_match,
            sel_char_match,
            md,
            sel_md,
            unsel_scrollbar,
        }
    }
    /// build an input field with the application's skin
    pub fn make_input() -> InputField {
        let mut input_field = InputField::default();
        input_field.set_normal_style(CompoundStyle::with_fgbg(ansi(230), gray(0)));
        //input_field.set_unfocused_style(CompoundStyle::with_fgbg(
        input_field
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
