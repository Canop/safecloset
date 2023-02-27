use {
    super::*,
    crokey::key,
    std::fmt,
};

#[derive(Clone)]
pub struct ConfirmOption {
    value: Option<String>,
}

#[allow(dead_code)]
impl ConfirmOption {
    pub fn confirmed(&self) -> bool {
        self.value.is_some()
    }
}

impl fmt::Display for ConfirmOption {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match &self.value {
            Some(s) => write!(f, "{s}"),
            None => write!(f, "Cancel"),
        }
    }
}

#[allow(dead_code)]
pub fn confirm<S1, S2>(
    intro: S1,
    question: S2,
) -> Menu<ConfirmOption>
where
    S1: Into<String>,
    S2: Into<String>,
{
    let mut menu = Menu::new();
    menu.add_item(
        ConfirmOption {
            value: Some(question.into()),
        },
        Some(key!(y)),
    );
    menu.add_item(ConfirmOption { value: None }, Some(key!(n)));
    menu.state.set_intro(intro);
    menu
}

#[allow(dead_code)]
pub type ConfirmMenu = Menu<ConfirmOption>;
