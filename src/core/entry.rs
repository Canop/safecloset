use serde::{Deserialize, Serialize};

/// one of the socks in the drawer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Entry {
    pub name: String,
    pub value: String,
}

impl Entry {
    #[allow(dead_code)]
    pub fn new<N: Into<String>, V: Into<String>>(name: N, value: V) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.value.is_empty()
    }
}
