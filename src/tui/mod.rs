mod app_state;
mod cmd_result;
mod content_view;
mod drawer_state;
mod entry_state;
mod global_view;
mod keys;
mod password_input_state;
mod scroll;
mod status_view;
mod title_view;
mod app;
mod view;

use {
    crate::{core::Closet, error::SafeClosetError},
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
    content_view::*,
    drawer_state::*,
    entry_state::*,
    global_view::*,
    keys::*,
    password_input_state::*,
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
    debug!("TUI starts");
    let r = app::run(&mut w, closet, hide_values);
    debug!("TUI ends");
    w.queue(cursor::Show)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    r
}
