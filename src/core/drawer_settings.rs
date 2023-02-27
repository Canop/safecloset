use serde::{
    Deserialize,
    Serialize,
};

/// settings of a drawer, saved in the drawer
#[derive(Serialize, Deserialize, Default)]
pub struct DrawerSettings {
    /// whether to hide unselected entry values
    pub hide_values: bool,
    /// whether to show the whole content of all values
    #[serde(default)]
    pub open_all_values: bool,
}
