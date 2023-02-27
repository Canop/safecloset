use {
    crokey::*,
    once_cell::sync::Lazy,
};

pub static KEY_FORMAT: Lazy<KeyEventFormat> = Lazy::new(|| {
    KeyEventFormat::default()
        .with_implicit_shift()
        .with_control("^")
});
