use {
    termimad::{InputField},
};

/// State of the application related to entries.
///
/// There's at most one entry state, referring to
/// the current selection or edition.
pub enum EntryState {
    NoneSelected,
    NameSelected { idx: usize },
    ValueSelected { idx: usize },
    NameEdit { idx: usize, input: InputField },
    ValueEdit { idx: usize, input: InputField },
}

impl EntryState {
    /// Return the index of the currently selected or
    /// edited entry, if any.
    pub fn idx(&self) -> Option<usize> {
        match self {
            Self::NoneSelected => None,
            Self::NameSelected { idx } => Some(*idx),
            Self::ValueSelected { idx } => Some(*idx),
            Self::NameEdit { idx, .. } => Some(*idx),
            Self::ValueEdit { idx, .. } => Some(*idx),
        }
    }
    pub fn is_name_selected(&self, entry_idx: usize) -> bool {
        match self {
            Self::NameSelected { idx } => *idx == entry_idx,
            _ => false,
        }
    }
    pub fn is_value_selected(&self, entry_idx: usize) -> bool {
        match self {
            Self::ValueSelected { idx } => *idx == entry_idx,
            _ => false,
        }
    }
    /// return the input editing the name of the entry
    /// of given index, if it's currently edited
    pub fn name_input(&mut self, entry_idx: usize) -> Option<&mut InputField> {
        match self {
            Self::NameEdit { idx, input } if *idx == entry_idx => Some(input),
            _ => None,
        }
    }
    /// return the input editing the value of the entry
    /// of given index, if it's currently edited
    pub fn value_input(&mut self, entry_idx: usize) -> Option<&mut InputField> {
        match self {
            Self::ValueEdit { idx, input } if *idx == entry_idx => Some(input),
            _ => None,
        }
    }
}
