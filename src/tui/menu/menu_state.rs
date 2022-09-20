use {
    crate::tui::*,
    crokey::key,
    crokey::crossterm::event::{KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    termimad::Area,
};

pub struct MenuItem {
    pub action: Action,
    pub area: Option<Area>,
}

#[derive(Default)]
pub struct MenuState {
    pub items: Vec<MenuItem>,
    pub selection: usize,
    pub scroll: usize,
}

impl MenuState {
    pub fn add_item(&mut self, action: Action) {
        self.items.push(MenuItem { action, area: None });
    }
    pub fn clear_item_areas(&mut self) {
        for item in self.items.iter_mut() {
            item.area = None;
        }
    }
    pub fn select(&mut self, selection: usize) {
        self.selection = selection.min(self.items.len());
    }
    pub(crate) fn fix_scroll(&mut self, page_height: usize) {
        let len = self.items.len();
        let sel = self.selection;
        if len <= page_height || sel < 3 ||  sel <= page_height / 2 {
            self.scroll = 0;
        } else if sel + 3 >= len {
            self.scroll = len - page_height;
        } else {
            self.scroll = (sel - 2).min(len - page_height);
        }
    }
    /// Handle a key event (not triggering the actions on their keys, only apply
    /// the menu mechanics)
    pub fn on_key(&mut self, key: KeyEvent) -> Option<Action> {
        let items = &self.items;
        if key == key!(down) {
            self.selection = (self.selection + 1) % items.len();
        } else if key == key!(up) {
            self.selection = (self.selection + items.len() - 1) % items.len();
        } else if key == key!(enter) {
            return Some(items[self.selection].action);
        }
        Action::for_key(key)
    }
    pub fn item_idx_at(&self, x: u16, y: u16) -> Option<usize> {
        for (idx, item) in self.items.iter().enumerate() {
            if let Some(area) = &item.area {
                if area.contains(x, y) {
                    return Some(idx);
                }
            }
        }
        None
    }
    /// handle a mouse event, returning the triggered action if any (on
    /// double click only)
    pub fn on_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        double_click: bool,
    ) -> Option<Action> {
        let is_click = matches!(
            mouse_event.kind,
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left),
        );
        if is_click {
            if let Some(selection) = self.item_idx_at(mouse_event.column, mouse_event.row) {
                self.selection = selection;
                if double_click {
                    return Some(self.items[self.selection].action);
                }
            }
        }
        None
    }
}

