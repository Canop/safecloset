use {
    super::*,
    crokey::crossterm::style::{
        Color,
        Color::*,
    },
    termimad::*,
};

/// Contain all the styling of the application
pub struct AppSkin {
    pub title: MadSkin,
    pub help: MadSkin,
    pub content: ContentSkin,
    pub dialog: DialogSkin,
    pub status: StatusSkin,
}

impl Default for AppSkin {
    fn default() -> Self {
        let title = make_title_skin();
        let help = make_help_skin();
        let content = ContentSkin::default();
        let dialog = DialogSkin::default();
        let status = StatusSkin::default();
        Self {
            title,
            help,
            content,
            dialog,
            status,
        }
    }
}

fn make_title_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.paragraph.set_fgbg(AnsiValue(252), AnsiValue(239));
    skin.bold.set_fg(ansi(222));
    skin.italic = CompoundStyle::with_fg(AnsiValue(222));
    skin.inline_code.set_bg(gray(2));
    skin
}

fn make_help_skin() -> MadSkin {
    let mut help = MadSkin::default();
    help.set_bg(gray(2));
    help.set_fg(ansi(230));
    help.set_headers_fg(ansi(222));
    help.italic = CompoundStyle::with_fg(Color::AnsiValue(222));
    help
}
