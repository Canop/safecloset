
#[derive(Debug, Clone, Copy)]
pub enum PasswordDialogPurpose {
    NewDrawer { depth: usize },
    OpenDrawer,
    ChangeDrawerPassword,
}

