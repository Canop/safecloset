/// The result of handling an event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdResult {
    Stay,
    Quit,
}

impl Default for CmdResult {
    fn default() -> Self {
        Self::Stay
    }
}

impl CmdResult {
    pub fn quit(self) -> bool {
        self == Self::Quit
    }
}
