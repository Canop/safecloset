use super::*;

/// An open drawer, with clear content.
///
/// Note that a drawer can't be saved while it's open.
/// Or more precisely, if you open a drawer, then try
/// to close it twice, it will fail.
/// For this reason, an open drawer isn't clonable.
pub struct OpenDrawer {
    /// kept so we can reencrypt
    pub(super) password: String,

    /// the index of the drawer among current drawer
    /// (makes sense only for this session as we'll scramble
    /// drawers on save)
    pub(super) drawer_idx: usize,

    /// the clear content
    pub entries: Vec<Entry>,

    /// user settings related to that drawer
    pub settings: DrawerSettings,

    /// an id preventing some internal API misuses
    /// (that a drawer isn't closed twice without
    /// reopening in between, for example)
    pub(super) open_id: usize,
}

impl OpenDrawer {
    /// Return the index of the first empty entry, creating
    /// it if necessary
    pub fn empty_entry(&mut self) -> usize {
        for (idx, entry) in self.entries.iter().enumerate() {
            if entry.is_empty() {
                return idx;
            }
        }
        self.entries.push(Entry::default());
        self.entries.len() - 1
    }
}

