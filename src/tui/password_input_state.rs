use {
    super::ContentSkin,
    termimad::InputField,
};

pub struct PasswordInputState {
    pub input: InputField,
}

impl PasswordInputState {
    pub fn new(hide_chars: bool) -> Self {
        let mut input = ContentSkin::make_input();
        input.password_mode = hide_chars;
        Self { input }
    }
}



