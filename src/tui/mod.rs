mod action;
mod app;
mod app_state;
mod cmd_result;
mod comments_editor;
mod content_view;
mod dialog;
mod drawer_drawing_layout;
mod drawer_focus;
mod drawer_state;
mod global_view;
mod help;
mod help_content;
mod keys;
mod matched_string;
mod menu;
mod message;
mod password_dialog;
mod scroll;
mod search_state;
mod skin;
mod status_view;
mod task;
mod title_view;
mod view;

use {
    crate::{
        cli::Args,
        core::OpenCloset,
        error::SafeClosetError,
    },
    crokey::crossterm::{
        cursor,
        event::{
            DisableMouseCapture,
            EnableMouseCapture,
        },
        terminal::{
            EnterAlternateScreen,
            LeaveAlternateScreen,
        },
        QueueableCommand,
    },
    std::{
        io::Write,
        time::Duration,
    },
};

pub(crate) use {
    action::*,
    app_state::*,
    cmd_result::*,
    comments_editor::*,
    content_view::*,
    dialog::*,
    drawer_drawing_layout::*,
    drawer_focus::*,
    drawer_state::*,
    global_view::*,
    help::*,
    help_content::*,
    keys::*,
    matched_string::*,
    menu::*,
    message::*,
    password_dialog::*,
    scroll::*,
    search_state::*,
    skin::*,
    status_view::*,
    task::*,
    title_view::*,
    view::*,
};

pub const MAX_INACTIVITY: Duration = Duration::from_secs(120);

pub trait ScreenWriter {
    fn go_to(
        &mut self,
        x: u16,
        y: u16,
    ) -> Result<(), SafeClosetError>;
}

/// the type used by all TUI writing functions
pub type W = std::io::BufWriter<std::io::Stdout>;

/// return the writer used by the application
fn writer() -> W {
    std::io::BufWriter::new(std::io::stdout())
}

impl ScreenWriter for W {
    fn go_to(
        &mut self,
        x: u16,
        y: u16,
    ) -> Result<(), SafeClosetError> {
        self.queue(cursor::MoveTo(x, y))?;
        Ok(())
    }
}

pub fn run(
    open_closet: OpenCloset,
    args: &Args,
) -> Result<(), SafeClosetError> {
    let mut w = writer();
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::Hide)?;
    w.queue(EnableMouseCapture)?;
    let r = app::run(&mut w, open_closet, args);
    w.queue(DisableMouseCapture)?;
    w.queue(cursor::Show)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    r
}
