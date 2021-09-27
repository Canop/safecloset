mod args;

pub use args::Args;

use crate::{
    core::OpenCloset,
    error::SafeClosetError,
    tui,
};

/// run the command line application.
///
/// Starts the TUI if a path to a closet is given
pub fn run() -> Result<(), SafeClosetError> {
    let args: Args = argh::from_env();
    if args.version {
        println!("SafeCloset {}", env!("CARGO_PKG_VERSION"),);
        return Ok(());
    }
    info!("args: {:#?}", &args);

    if let Some(path) = &args.path {
        let closet = OpenCloset::open_or_create(path.clone())?;
        tui::run(closet, &args)?;
    } else {
        println!(
            "Please provide as argument the path to the closet file to create or open, \
            or use --help for help."
        );
    }

    Ok(())
}
