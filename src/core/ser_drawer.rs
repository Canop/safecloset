use {
    super::*,
    serde::{Deserialize, Serialize},
    std::ops::Range,
};

pub const GARBAGE_SIZE: Range<usize> = 10_000..30_000;

/// Serializable Drawer
///
/// The ser_drawer exists only the time of the serialization
/// and encryption, and the time of deserialization for drawer
/// opening
#[derive(Serialize, Deserialize)]
pub struct SerDrawer {
    entries: Vec<Entry>,
    settings: DrawerSettings,
    garbage: Box<[u8]>,
}

impl SerDrawer {
    pub fn new(
        open_drawer: OpenDrawer,
    ) -> Self {
        Self {
            entries: open_drawer.entries,
            settings: open_drawer.settings,
            garbage: random_bytes_random_size(GARBAGE_SIZE),
        }
    }

    pub fn into_open_drawer(
        self,
        drawer_idx: usize,
        password: String,
        open_id: usize,
    ) -> OpenDrawer {
        OpenDrawer {
            entries: self.entries,
            settings: self.settings,
            drawer_idx,
            password,
            open_id,
        }
    }
}
