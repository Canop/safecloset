
/// a potentially long task, which is queued before execution
pub enum Task {
    Save,
    CreateDrawer(String),
    OpenDrawer(String),
    CloseDrawer,
    ChangePassword(String),
}

impl Task {
    /// return the label to display during task execution
    pub fn label(&self) -> &'static str {
        match self {
            Self::Save => "Saving...",
            Self::CreateDrawer(_) => "Creating a drawer...",
            Self::OpenDrawer(_) => "Opening...",
            Self::CloseDrawer => "Closing...",
            Self::ChangePassword(_) => "Changing password...",
        }
    }
}
