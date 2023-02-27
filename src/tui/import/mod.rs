mod choices;
mod import_state;
mod import_view;

pub use {
    choices::*,
    import_state::*,
    import_view::*,
};

use {
    super::*,
    crokey::crossterm::event::{
        KeyEvent,
        MouseEvent,
    },
    std::path::PathBuf,
    termimad::Area,
};

pub struct Import {
    state: ImportState,
    view: ImportView,
}

impl Import {
    pub fn new(
        dst_path: PathBuf,
        dst_drawer: DrawerState,
    ) -> Self {
        let state = ImportState::new(dst_path, dst_drawer);
        let view = ImportView::default();
        Self { state, view }
    }
    pub fn toggle_hide_chars(&mut self) {
        self.state.toggle_hide_chars();
    }
    pub fn on_key(
        &mut self,
        key: KeyEvent,
    ) -> bool {
        self.state.apply_key_event(key)
    }
    pub fn on_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        double_click: bool,
    ) {
        self.state.on_mouse_event(mouse_event, double_click);
    }
    pub fn set_available_area(
        &mut self,
        area: Area,
    ) {
        self.view.set_available_area(area);
    }
    pub fn draw(
        &mut self,
        w: &mut W,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        self.view.draw(w, &mut self.state, app_skin)
    }
    pub fn status(&self) -> &'static str {
        self.state.status()
    }
    pub fn take_back_drawer(self) -> DrawerState {
        self.state.dst_drawer_state
    }
    pub fn is_finished(&self) -> bool {
        self.state.is_finished()
    }
}
