use {
    crate::tui::*,
    crokey::KeyCombination,
    termimad::Area,
};

pub struct Menu<I> {
    pub state: MenuState<I>,
    view: MenuView,
}

impl<I: ToString + Clone> Menu<I> {
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
    pub fn set_available_area(
        &mut self,
        area: Area,
    ) {
        <MenuView as View<MenuState<I>>>::set_available_area(&mut self.view, area);
    }
    pub fn set_intro<S: Into<String>>(
        &mut self,
        intro: S,
    ) {
        self.state.set_intro(intro);
    }
    pub fn add_item(
        &mut self,
        action: I,
        key: Option<KeyCombination>,
    ) {
        self.state.add_item(action, key);
    }
}
