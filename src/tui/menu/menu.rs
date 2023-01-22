use {
    crate::tui::*,
    termimad::Area,
};



pub struct Menu<I> {
    pub state: MenuState<I>,
    view: MenuView<I>,
}

pub type ActionMenu = Menu<Action>;


impl<I: ToString> Menu<I> {
    pub fn new() -> Self {
        Self {
            state: Default::default(),
            view: Default::default(),
        }
    }
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
    pub fn add_item(&mut self, action: I) {
        self.state.add_item(action);
    }
}
