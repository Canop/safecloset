use {
    super::*,
    crate::{
        core::Closet,
        error::SafeClosetError,
    },
    termimad::{Area, Event, EventSource},
};

/// Run the Terminal User Interface until the user decides to quit.
///
/// The terminal must be already in alternate and raw mode
pub(crate) fn run(w: &mut W, closet: Closet) -> Result<(), SafeClosetError> {
    let mut state = AppState::new(closet);
    let mut view = GlobalView::default();
    view.set_area(Area::full_screen());
    view.draw(w, &mut state)?;
    let event_source = EventSource::new()?;
    let mut quit = false;
    for event in event_source.receiver() {
        match event {
            Event::Resize(width, height) => {
                view.set_area(Area::new(0, 0, width, height));
            }
            Event::Key(key) => {
                let cmd_result = state.on_key(key)?;
                if cmd_result.quit() {
                    debug!("user requests quit");
                    quit = true;
                }
            }
            _ => {}
        }
        event_source.unblock(quit);
        if quit {
            break;
        }
        view.draw(w, &mut state)?;
    }
    Ok(())
}
