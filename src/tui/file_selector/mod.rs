mod file_selector_state;
mod file_selector_view;
mod file_type;

pub use {
    file_selector_state::*,
    file_selector_view::*,
    file_type::*,
};

use {
    super::*,
    crokey::crossterm::event::{
        KeyEvent,
        MouseEvent,
    },
    std::path::Path,
};

pub struct FileSelector {
    state: FileSelectorState,
    pub view: FileSelectorView,
}

impl FileSelector {
    pub fn new(
        intro: String,
        file_type: FileType,
    ) -> Self {
        let state = FileSelectorState::new(intro, file_type);
        let view = FileSelectorView::default();
        Self { state, view }
    }
    pub fn get_selected_file(&self) -> Option<&Path> {
        self.state.get_selected_file()
    }
    pub fn apply_key_event(
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
    pub fn get_message(&self) -> &'static str {
        self.state.message
    }
    pub fn draw(
        &mut self,
        w: &mut W,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        self.view.draw(w, &mut self.state, app_skin)
    }
}
