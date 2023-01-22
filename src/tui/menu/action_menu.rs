use {
    super::Menu,
    crate::tui::Action,
};

pub type ActionMenu = Menu<Action>;

impl ActionMenu {
    pub fn add_action(&mut self, action: Action) {
        self.add_item(action, action.key());
    }
}
