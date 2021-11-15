use {
    crossterm::style::Color,
    termimad::*,
};

/// The skin for menus and dialogs
pub struct DialogSkin {
    pub md: MadSkin,
    pub sel_md: MadSkin,
}

impl Default for DialogSkin {
    fn default() -> Self {
        let mut md = MadSkin {
            italic: CompoundStyle::with_fg(Color::AnsiValue(222)),
            ..Default::default()
        };
        md.set_bg(gray(4));
        let mut sel_md = md.clone();
        sel_md.set_bg(gray(8));
        Self {
            md,
            sel_md,
        }
    }
}

