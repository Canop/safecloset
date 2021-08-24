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
    check_id: String,
    entries: Vec<Entry>,
    settings: DrawerSettings,
    garbage: Box<[u8]>,
}

impl SerDrawer {
    pub fn new(
        open_drawer: OpenDrawer,
        check_id: String,
    ) -> Self {
        Self {
            check_id,
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
        check_id: &str,
    ) -> Result<OpenDrawer, CoreError> {
        if self.check_id != check_id {
            // this check prevents the hypothetical (and almost impossible)
            // case of a drawer which would be decrypted then deserialized
            // in a "valid" drawer with a wrong key without error. This may
            // become more possible with a change in serialization format.
            return Err(CoreError::InvalidCheckId);
        }
        Ok(OpenDrawer {
            entries: self.entries,
            settings: self.settings,
            drawer_idx,
            password,
            open_id,
        })
    }
}
