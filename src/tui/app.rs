use {
    super::*,
    crate::{
        cli::Args,
        core::OpenCloset,
        error::SafeClosetError,
        timer::Timer,
    },
    crokey::crossterm::event::Event,
    crossbeam::select,
    termimad::{
        Area,
        EventSource,
    },
};

/// Run the Terminal User Interface until the user decides to quit.
///
/// The terminal must be already in alternate and raw mode
#[allow(unused_mut)]
pub(super) fn run(
    w: &mut W,
    open_closet: OpenCloset,
    args: &Args,
) -> Result<(), SafeClosetError> {
    let mut state = AppState::new(open_closet, args);
    let skin = AppSkin::default();
    let mut view = GlobalView::default();
    view.set_available_area(Area::full_screen());
    view.draw(w, &mut state, &skin)?;
    let event_source = EventSource::new()?;
    let events = event_source.receiver();
    let (timer, timer_rx) = Timer::new(MAX_INACTIVITY);
    loop {
        select! {
            // user events
            recv(events) -> timed_event => {
                let timed_event = timed_event?;
                let mut quit = false;
                match timed_event.event {
                    Event::Resize(mut width, mut height) => {
                        // I don't know why but Crossterm seems to always report an
                        // understimated size on Windows
                        #[cfg(windows)]
                        {
                            width += 1;
                            height += 1;
                        }
                        view.set_available_area(Area::new(0, 0, width, height));
                    }
                    Event::Key(key) => {
                        let cmd_result = state.on_key(key)?;
                        if cmd_result.quit() {
                            debug!("user requests quit");
                            quit = true;
                        }
                        timer.reset();
                    }
                    Event::Mouse(mouse_event) => {
                        state.on_mouse_event(mouse_event, timed_event.double_click)?;
                        timer.reset();
                    }
                }
                event_source.unblock(quit);
                if quit {
                    break;
                }
                view.draw(w, &mut state, &skin)?;
                while state.has_pending_task() {
                    let cmd_result = state.run_pending_task()?;
                    if cmd_result.quit() {
                        debug!("quit on end of pending task");
                        quit = true;
                    }
                    view.draw(w, &mut state, &skin)?;
                }
                if quit {
                    break;
                }
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
