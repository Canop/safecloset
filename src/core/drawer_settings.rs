use {
    serde::{Deserialize, Serialize},
};

/// settings of a drawer, saved in the drawer
#[derive(Serialize, Deserialize)]
pub struct DrawerSettings {
    /// whether to hide unselected entry values
    pub hide_values: bool,
}

impl Default for DrawerSettings {
    fn default() -> Self {
        Self {
            hide_values: false,
        }
    }
}
