mod comments_editor_state;
mod comments_editor_view;

pub use {
    comments_editor_state::*,
    comments_editor_view::*,
};

use {
    super::*,
    crokey::crossterm::event::{
        KeyEvent,
        MouseEvent,
    },
};

pub struct CommentsEditor {
    state: CommentsEditorState,
    pub view: CommentsEditorView,
}

impl CommentsEditor {
    pub fn new(comments: &str) -> Self {
        let state = CommentsEditorState::new(comments);
        let view = CommentsEditorView::default();
        Self { state, view }
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
    pub fn get_comments(&mut self) -> String {
        self.state.comments.get_content()
    }
    pub fn draw(
        &mut self,
        w: &mut W,
        app_skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        self.view.draw(w, &mut self.state, app_skin)
    }
}
