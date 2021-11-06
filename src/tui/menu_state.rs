
use {
    super::*,
    crossterm::{
        event::KeyEvent,
    },
};

#[derive(Default)]
pub struct MenuState {
    pub actions: Vec<Action>,
    pub selection: usize,
    pub scroll: usize,
}

impl MenuState {
    /// Handle a key event (not triggering the actions on their keys, only apply
    /// the menu mechanics)
    pub fn on_key(&mut self, key: KeyEvent) -> Option<Action> {
        let actions = &self.actions;
        if key == DOWN {
            self.selection = (self.selection + 1) % actions.len();
        } else if key == UP {
            self.selection = (self.selection + actions.len() - 1) % actions.len();
        } else if key == ENTER {
            return Some(actions[self.selection]);
        }
        None
    }
}

