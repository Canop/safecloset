use {
    super::*,
    serde::{Deserialize, Serialize},
};

/// What's inside a drawer
#[derive(Serialize, Deserialize)]
pub struct DrawerContent {

    pub id: DrawerId,

    /// the entries of this depth
    pub entries: Vec<Entry>,

    /// user settings related to that drawer
    pub settings: DrawerSettings,

    /// the crypted sub-drawers
    pub closet: Closet,

    /// some random bytes, rewritten before every save
    garbage: Box<[u8]>,
}

impl Identified for DrawerContent {
    fn get_id(&self) -> &DrawerId {
        &self.id
    }
}

impl DrawerContent {

    pub fn new(depth: usize) -> Result<Self, CoreError> {
        let id = DrawerId::new();
        let entries = Vec::new();
        let settings = DrawerSettings::default();
        let closet = Closet::new(depth + 1)?;
        let garbage = Vec::new().into(); // will be (re)filled for save
        Ok(Self {
            id,
            entries,
            settings,
            closet,
            garbage,
        })
    }

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

    /// Insert a new entry at the given index or before and
    /// return the index of the new entry
    #[allow(dead_code)]
    pub fn insert_before(&mut self, idx: usize) -> usize {
        let idx = if self.entries.is_empty() {
            0
        } else {
            idx.min(self.entries.len() - 1)
        };
        self.entries.insert(idx, Entry::default());
        idx
    }

    /// Insert a new entry after the new entry if possible.
    ///
    /// Return the index of the new entry.
    pub fn insert_after(&mut self, idx: Option<usize>) -> usize {
        let idx = match (idx, self.entries.is_empty()) {
            (Some(idx), false) => (idx+1).min(self.entries.len()),
            _ => 0,
        };
        self.entries.insert(idx, Entry::default());
        idx
    }

    /// Shuffle the drawers (thus ensuring the last created one
    /// isn't at the end), add some random bytes which makes the
    /// content's size undetectable
    pub fn add_noise(&mut self) {
        self.garbage = random_bytes_random_size(5..5000);
        self.closet.shuffle_drawers();
    }

    /// Remove entries with both name and value empty
    pub fn remove_empty_entries(&mut self) {
        self.entries.retain(|e| !e.is_empty());
    }
}

