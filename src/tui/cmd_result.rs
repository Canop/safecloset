/// The result of handling an event
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CmdResult {
    #[default]
    Stay,
    Quit,
}

impl CmdResult {
    pub fn quit(self) -> bool {
        self == Self::Quit
    }
}
