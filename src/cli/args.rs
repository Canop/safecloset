use {
    argh::FromArgs,
    std::path::PathBuf,
};

#[derive(Debug, FromArgs)]
/// SafeCloset keeps your secrets -
/// Documentation and source code at https://dystroy.org/safecloset
pub struct Args {
    /// print the version
    #[argh(switch, short = 'v')]
    pub version: bool,

    /// hide unselected values
    #[argh(switch, short = 'h')]
    pub hide: bool,

    /// immediately prompt for a password to open a drawer
    #[argh(switch, short = 'o')]
    pub open: bool,

    #[argh(positional)]
    /// the closet file to open or create
    pub path: Option<PathBuf>,
}
