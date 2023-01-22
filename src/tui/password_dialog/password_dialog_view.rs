use {
    super::*,
    crate::tui::*,
    termimad::*,
};

#[derive(Default)]
pub struct PasswordDialogView {
    area: Area,
}

static MD_CREATE_TOP_DRAWER: &str = r#"Type the passphrase for the new top level drawer:"#;
static MD_CREATE_DEEP_DRAWER: &str = r#"Type the passphrase for this deep drawer (to create a top level drawer, cancel then close the drawer you're in):"#;
static MD_OPEN_TOP_DRAWER: &str = r#"Type the passphrase of the shallow drawer you want to open:"#;
static MD_OPEN_DEEP_DRAWER: &str = r#"Type the passphrase of the deep drawer you want to open:"#;
static MD_CHANGE_PASSWORD: &str = r#"Type the new passphrase (the previous version will still be available in a '.old' backup file after you save once):"#;
static MD_HIDDEN_CHARS: &str = r#"Characters are hidden. Type *^h* to toggle visibility."#;
static MD_VISIBLE_CHARS: &str = r#"Characters are visible. Type *^h* to hide them."#;

const INTERNAL_HEIGHT: u16 =
    3    // intro: 3
    + 2  // pwd: 2
    + 3; // char hiding text: 3

impl PasswordDialogView {
    fn introduction_text(state: &PasswordDialogState) -> &'static str {
        match state.purpose {
            PasswordDialogPurpose::NewDrawer { depth } => {
                if depth > 0 {
                    MD_CREATE_DEEP_DRAWER
                } else {
                    MD_CREATE_TOP_DRAWER
                }
            }
            PasswordDialogPurpose::OpenDrawer { depth } => {
                if depth > 0 {
                    MD_OPEN_DEEP_DRAWER
                } else {
                    MD_OPEN_TOP_DRAWER
                }
            }
            PasswordDialogPurpose::ChangeDrawerPassword => MD_CHANGE_PASSWORD,
        }
    }
}

impl View<PasswordDialogState> for PasswordDialogView {

    fn set_available_area(&mut self, mut area: Area) {
        if area.width > 60 && area.height > 8 {
            let hw = area.width / 2;
            let dhw = (hw * 3 / 4).min(hw - 2);
            area.left = hw - dhw;
            area.width = 2 * dhw;
            let h = INTERNAL_HEIGHT + 2;
            area.top += (area.height - h) / 3;
            area.height = h;
        }
        self.area = area;
    }

    /// Render the view in its area
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut PasswordDialogState, // mutable to allow adapt to terminal size changes
        skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {

        // border
        let border_colors = skin.dialog.md.table.compound_style.clone();
        let area = &self.area;
        let mut rect = Rect::new(area.clone(), border_colors);
        rect.set_fill(true);
        rect.set_border_style(BORDER_STYLE_BLAND);
        rect.draw(w)?;

        // introduction
        let mut area = Area::new(area.left + 1, area.top + 1, area.width - 2, 3);
        let text = Self::introduction_text(state);
        skin.dialog.md.write_in_area_on(w, text, &area)?;

        // password input
        area.top += 3;
        state.password.change_area(area.left, area.top, area.width);
        state.password.display_on(w)?;

        // chars hiding
        area.top += 2;
        let tip = if state.password.password_mode {
            MD_HIDDEN_CHARS
        } else {
            MD_VISIBLE_CHARS
        };
        skin.dialog.md.write_in_area_on(w, tip, &area)?;

        Ok(())
    }
}


