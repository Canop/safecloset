use {
    termimad::{Area, InputField},
};

pub struct PasswordInputState {
    pub input: InputField,
}

impl PasswordInputState {
    pub fn new(hide_chars: bool) -> Self {
        let mut input = InputField::new(Area::uninitialized());
        input.password_mode = hide_chars;
        Self { input }
    }
}



