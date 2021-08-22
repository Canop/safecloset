use serde::{Deserialize, Serialize};

/// one of the socks in the drawer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Entry {
    pub name: String,
    pub value: String,
}

impl Entry {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.value.is_empty()
    }
}
