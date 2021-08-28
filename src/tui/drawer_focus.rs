use {
    termimad::{InputField},
};

/// Where the focus of the user/app is in the drawer,
/// and the related data.
pub enum DrawerFocus {
    NoneSelected,
    PendingRemoval { line: usize },
    NameSelected { line: usize },
    ValueSelected { line: usize },
    NameEdit { line: usize, input: InputField },
    ValueEdit { line: usize, input: InputField },
    SearchEdit,
}

impl DrawerFocus {
    /// Return the index of the currently selected or
    /// edited entry, if any.
    pub fn line(&self) -> Option<usize> {
        match self {
            Self::NoneSelected => None,
            Self::PendingRemoval { line } => Some(*line),
            Self::NameSelected { line } => Some(*line),
            Self::ValueSelected { line } => Some(*line),
            Self::NameEdit { line, .. } => Some(*line),
            Self::ValueEdit { line, .. } => Some(*line),
            Self::SearchEdit => None,
        }
    }
    pub fn is_pending_removal(&self) -> bool {
        matches!(self, DrawerFocus::PendingRemoval { .. })
    }
    pub fn is_name_selected(&self, entry_line: usize) -> bool {
        match self {
            Self::NameSelected { line } => *line == entry_line,
            _ => false,
        }
    }
    pub fn is_value_selected(&self, entry_line: usize) -> bool {
        match self {
            Self::ValueSelected { line } => *line == entry_line,
            _ => false,
        }
    }
    /// return the input editing the name of the entry
    /// of given index, if it's currently edited
    pub fn name_input(&mut self, entry_line: usize) -> Option<&mut InputField> {
        match self {
            Self::NameEdit { line, input } if *line == entry_line => Some(input),
            _ => None,
        }
    }
    /// return the input editing the value of the entry
    /// of given index, if it's currently edited
    pub fn value_input(&mut self, entry_line: usize) -> Option<&mut InputField> {
        match self {
            Self::ValueEdit { line, input } if *line == entry_line => Some(input),
            _ => None,
        }
    }
}
