mod app_state;
mod cmd_result;
mod content_skin;
mod content_view;
mod drawer_state;
mod drawer_edit_state;
mod drawer_focus;
mod global_view;
mod keys;
mod matched_string;
mod password_input_state;
mod search_state;
mod scroll;
mod status_view;
mod title_view;
mod app;
mod view;

use {
    crate::{
        core::Closet,
        error::SafeClosetError,
    },
    crossterm::{
        self, cursor,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    std::io::Write,
};

pub(crate) use {
    app_state::*,
    cmd_result::*,
    content_skin::*,
    content_view::*,
    drawer_state::*,
    drawer_edit_state::*,
    drawer_focus::*,
    global_view::*,
    keys::*,
    matched_string::*,
    password_input_state::*,
    search_state::*,
    scroll::*,
    status_view::*,
    title_view::*,
    view::*,
};

/// the type used by all TUI writing functions
pub type W = std::io::BufWriter<std::io::Stdout>;

/// return the writer used by the application
fn writer() -> W {
    std::io::BufWriter::new(std::io::stdout())
}

pub fn run(
    closet: Closet,
    hide_values: bool,
) -> Result<(), SafeClosetError> {
    let mut w = writer();
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::Hide)?;
    let r = app::run(&mut w, closet, hide_values);
    w.queue(cursor::Show)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    r
}
