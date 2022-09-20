use {
    crokey::crossterm::{
        style::{
            Attribute,
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

    normal_styles: Styles,
    selected_styles: Styles,
    faded_styles: Styles,

}

#[derive(Clone)]
pub struct Styles {
    /// skin for all the markdown parts
    pub md: MadSkin,

    /// style of pattern matching chars
    pub char_match: CompoundStyle,
}

impl Default for ContentSkin {
    fn default() -> Self {
        let bg = gray(2);
        let mut md = MadSkin::default();
        md.set_fg(ansi(230));
        md.bold = CompoundStyle::new(
            Some(AnsiValue(222)),
            None,
            Attribute::Bold.into(),
        );
        md.italic = CompoundStyle::with_fg(AnsiValue(222));
        md.set_bg(bg);
        let char_match_fg = AnsiValue(41);
        let char_match = CompoundStyle::with_fgbg(char_match_fg, bg);
        let normal_styles = Styles { md, char_match };

        let sel_bg = gray(5);
        let mut selected_styles = normal_styles.clone();
        selected_styles.md.set_bg(sel_bg);
        selected_styles.md.scrollbar.thumb.set_fg(gray(10));
        selected_styles.md.scrollbar.track.set_fg(gray(5));
        selected_styles.char_match = CompoundStyle::with_fgbg(char_match_fg, sel_bg);

        let mut faded_styles = normal_styles.clone();
        faded_styles.md.blend_with(bg, 0.6);

        Self {
            bg,
            normal_styles,
            selected_styles,
            faded_styles,
        }
    }
}

impl ContentSkin {
    /// build an input field with the application's skin
    pub fn make_input() -> InputField {
        let mut input_field = InputField::default();
        input_field.set_normal_style(CompoundStyle::with_fgbg(ansi(230), gray(0)));
        input_field
    }
    pub fn styles(&self, selected: bool, faded: bool) -> &Styles {
        if faded {
            &self.faded_styles
        } else if selected {
            &self.selected_styles
        } else {
            &self.normal_styles
        }
    }
    pub fn match_style(&self, selected: bool, faded: bool) -> &CompoundStyle {
        &self.styles(selected, faded).char_match
    }
    pub fn txt_style(&self, selected: bool, faded: bool) -> &CompoundStyle {
        &self.styles(selected, faded).md.paragraph.compound_style
    }
    pub fn tbl_style(&self, selected: bool, faded: bool) -> &CompoundStyle {
        &self.styles(selected, faded).md.table.compound_style
    }
    pub fn scrollbar_style(&self, selected: bool, faded: bool) -> &ScrollBarStyle {
        &self.styles(selected, faded).md.scrollbar
    }
}
