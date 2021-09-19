mod args;

use crate::{
    core::OpenCloset,
    error::SafeClosetError,
    tui,
};

/// run the command line application.
///
/// Starts the TUI if a path to a closet is given
pub fn run() -> Result<(), SafeClosetError> {
    let args: args::Args = argh::from_env();
    if args.version {
        println!("SafeCloset {}", env!("CARGO_PKG_VERSION"),);
        return Ok(());
    }
    info!("args: {:#?}", &args);

    if let Some(path) = args.path {
        let closet = OpenCloset::open_or_create(path)?;
        tui::run(closet, args.hide)?;
    } else {
        println!("Please provide as argument the path to the closet file to create or open");
    }

    Ok(())
}
