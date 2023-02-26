use {
    super::*,
    crate::tui::ContentSkin,
    crokey::crossterm::event::{
        KeyEvent,
        MouseEvent,
    },
    std::path::{
        Path,
        PathBuf,
    },
    termimad::*,
};

pub struct FileSelectorState {
    pub intro: String,
    pub file_type: FileType,
    pub input: InputField,
    pub path: Option<PathBuf>, // The result, if any
    pub message: &'static str,
}

impl FileSelectorState {
    pub fn new(
        intro: String,
        file_type: FileType,
    ) -> Self {
        let input = ContentSkin::make_input();
        let path = None;
        let message = file_type.check(&PathBuf::new()).message;
        Self {
            intro,
            file_type,
            input,
            path,
            message,
        }
    }
    pub fn get_selected_file(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    fn update_path(&mut self) {
        let path: PathBuf = self.input.get_content().into();
        let check = self.file_type.check(&path);
        self.path = check.ok.then_some(path);
        self.message = check.message;
    }
    pub fn apply_key_event(
        &mut self,
        key: KeyEvent,
    ) -> bool {
        let b = self.input.apply_key_event(key);
        self.update_path();
        b
    }
    /// handle a mouse event
    pub fn on_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        double_click: bool,
    ) {
        self.input.apply_mouse_event(mouse_event, double_click);
        self.update_path();
    }
}
