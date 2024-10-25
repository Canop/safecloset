use {
    crokey::crossterm::style::Color::*,
    termimad::{
        CompoundStyle,
        MadSkin,
        ansi,
        gray,
    },
};

pub struct StatusSkin {
    pub hint: MadSkin,
    pub info: MadSkin,
    pub task: MadSkin,
    pub error: MadSkin,
}

impl Default for StatusSkin {
    fn default() -> Self {
        let mut hint = MadSkin::default();
        hint.paragraph.set_fgbg(AnsiValue(252), AnsiValue(239));
        hint.italic = CompoundStyle::with_fg(AnsiValue(222));
        let mut info = MadSkin::default();
        info.paragraph.set_fg(AnsiValue(252));
        info.italic = CompoundStyle::with_fg(AnsiValue(222));
        info.set_bg(ansi(24));
        let mut task = MadSkin::default();
        task.paragraph.set_fg(gray(1));
        task.set_bg(ansi(222));
        let mut error = MadSkin::default();
        error.paragraph.set_fgbg(AnsiValue(254), AnsiValue(160));
        Self {
            hint,
            info,
            task,
            error,
        }
    }
}
