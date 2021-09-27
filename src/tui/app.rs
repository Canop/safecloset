use {
    super::*,
    crate::{
        cli::Args,
        core::OpenCloset,
        error::SafeClosetError,
        timer::Timer,
    },
    crossterm::event::KeyModifiers,
    crossbeam::select,
    termimad::{Area, Event, EventSource},
};


/// Run the Terminal User Interface until the user decides to quit.
///
/// The terminal must be already in alternate and raw mode
pub(super) fn run(
    w: &mut W,
    open_closet: OpenCloset,
    args: &Args,
) -> Result<(), SafeClosetError> {
    let mut state = AppState::new(open_closet, args);
    let mut view = GlobalView::default();
    view.set_area(Area::full_screen());
    view.draw(w, &mut state)?;
    let event_source = EventSource::new()?;
    let events = event_source.receiver();
    let (timer, timer_rx) = Timer::new(MAX_INACTIVITY);
    let mut quit = false;
    loop {
        select! {

            // user events
            recv(events) -> event => {
                match event? {
                    Event::Resize(width, height) => {
                        view.set_area(Area::new(0, 0, width, height));
                    }
                    Event::Key(key) => {
                        let cmd_result = state.on_key(key)?;
                        if cmd_result.quit() {
                            debug!("user requests quit");
                            quit = true;
                        }
                        timer.reset();
                    }
                    Event::Click(x, y, KeyModifiers::NONE) => {
                        state.on_click(x, y)?;
                        timer.reset();
                    }
                    _ => {}
                }
                event_source.unblock(quit);
                if quit {
                    break;
                }
                view.draw(w, &mut state)?;
            }

            // timer (so that safecloset doesn't stay open
            // if you quit your PC)
            recv(timer_rx) -> ring => {
                info!("Inactivity detection, quitting (delay: {:?})", MAX_INACTIVITY);
                debug!("ring type: {:?}", ring);
                event_source.unblock(true);
                break;
            }

        }
    }
    Ok(())
}
