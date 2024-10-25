use {
    crate::tui::ContentSkin,
    crokey::{
        crossterm::event::{
            KeyEvent,
            MouseEvent,
        },
        key,
    },
    termimad::*,
};

pub struct CommentsEditorState {
    pub comments: InputField,
}

impl CommentsEditorState {
    pub fn new(content: &str) -> Self {
        let mut comments = ContentSkin::make_input();
        comments.new_line_on(key!(alt - enter));
        comments.new_line_on(key!(ctrl - enter));
        comments.set_str(content);
        Self { comments }
    }
    pub fn apply_key_event(
        &mut self,
        key: KeyEvent,
    ) -> bool {
        self.comments.apply_key_event(key)
    }
    /// handle a mouse event
    pub fn on_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        double_click: bool,
    ) {
        self.comments.apply_mouse_event(mouse_event, double_click);
    }
}
