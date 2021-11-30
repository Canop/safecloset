use {
    std::fmt,
    termimad::{InputField},
};

/// Where the focus of the user/app is in the drawer,
/// and the related data.
pub enum DrawerFocus {
    NoneSelected,
    NameSelected { line: usize },
    ValueSelected { line: usize },
    NameEdit { line: usize, input: InputField },
    ValueEdit { line: usize, input: InputField },
    SearchEdit { previous_idx: Option<usize> },
    PendingRemoval { line: usize },
}

impl DrawerFocus {
    /// Return the index of the currently selected or
    /// edited entry, if any.
    pub fn line(&self) -> Option<usize> {
        match self {
            Self::NoneSelected => None,
            Self::NameSelected { line } => Some(*line),
            Self::ValueSelected { line } => Some(*line),
            Self::NameEdit { line, .. } => Some(*line),
            Self::ValueEdit { line, .. } => Some(*line),
            Self::SearchEdit { .. } => None,
            Self::PendingRemoval { line } => Some(*line),
        }
    }
    /// Return the mutable selection, when it's mutable
    /// (ie not for the edit as the input fields would
    /// hold data unrelated to the selection)
    pub fn selection_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::NameSelected { line } => Some(line),
            Self::ValueSelected { line } => Some(line),
            _ => None,
        }
    }
    pub fn is_search(&self) -> bool {
        matches!(self, DrawerFocus::SearchEdit { .. })
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
    pub fn is_line_pending_removal(&self, entry_line: usize) -> bool {
        match self {
            Self::PendingRemoval { line } => *line == entry_line,
            _ => false,
        }
    }
    pub fn is_value_selected(&self, entry_line: usize) -> bool {
        match self {
            Self::ValueSelected { line } => *line == entry_line,
            _ => false,
        }
    }
    pub fn is_entry_edit(&self) -> bool {
        matches!(self, DrawerFocus::NameEdit { .. } | DrawerFocus::ValueEdit { .. })
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

impl fmt::Debug for DrawerFocus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoneSelected => {
                f.debug_struct("NoneSelected").finish()
            }
            Self::NameSelected { line } => {
		f.debug_struct("NameSelected").field("line", line).finish()
            }
            Self::ValueSelected { line } => {
		f.debug_struct("ValueSelected").field("line", line).finish()
            }
            Self::NameEdit { line, .. } => {
		f.debug_struct("NameEdit").field("line", line).finish()
            }
            Self::ValueEdit { line, .. } => {
		f.debug_struct("ValueEdit").field("line", line).finish()
            }
            Self::SearchEdit { .. } => {
		f.debug_struct("SearchEdit").finish()
            }
            Self::PendingRemoval { line } => {
		f.debug_struct("PendingRemoval").field("line", line).finish()
            }
        }
    }
}

