use {
    crokey::*,
    once_cell::sync::Lazy,
};

pub static KEY_FORMAT: Lazy<KeyCombinationFormat> = Lazy::new(|| {
    KeyCombinationFormat::default()
        .with_implicit_shift()
        .with_control("^")
});
