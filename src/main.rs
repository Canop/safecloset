mod cli;
mod core;
mod error;
mod search;
mod tui;

#[macro_use]
extern crate cli_log;

fn main() -> Result<(), error::SafeClosetError> {
    init_cli_log!();
    cli::run()?;
    info!("bye");
    Ok(())
}
