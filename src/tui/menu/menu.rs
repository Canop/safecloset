use {
    crate::tui::*,
    termimad::Area,
};

#[derive(Default)]
pub struct Menu {
    pub state: MenuState,
    view: MenuView,
}

impl Menu {
    pub fn draw(
        &mut self,
        w: &mut W,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        self.view.draw(w, &mut self.state, app_skin)
    }
    pub fn set_available_area(&mut self, area: Area) {
        self.view.set_available_area(area);
    }
    pub fn add_item(&mut self, action: Action) {
        self.state.actions.push(action);
    }
    pub fn select(&mut self, selection: usize) {
        self.state.selection = selection;
    }
}
