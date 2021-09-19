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

}

