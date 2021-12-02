use {
    super::*,
    crate::tui::*,
    termimad::*,
};

#[derive(Default)]
pub struct CommentsEditorView {
    area: Area,
}

static MD_BEFORE: &str = r#"Closet comments (not crypted):"#;
static MD_AFTER: &str = r#"Hit *^enter* or *alt-enter* to add a line, and *enter* to validate"#;

impl CommentsEditorView {
}

impl View for CommentsEditorView {

    type State = CommentsEditorState;

    fn set_available_area(&mut self, mut area: Area) {
        if area.width > 60 && area.height > 11 {
            area.left = 2;
            area.width -= 4;
            area.top += 3;
            area.height -= 6;
        }
        self.area = area;
    }

    /// Render the view in its area
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut Self::State, // mutable to allow adapt to terminal size changes
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
        let intro_area = Area::new(area.left + 1, area.top + 1, area.width - 2, 3);
        skin.dialog.md.write_in_area_on(w, MD_BEFORE, &intro_area)?;

        // comments textarea
        let text_area = Area::new(area.left + 1, area.top + 3, area.width -2, area.height - 7);
        state.comments.set_area(text_area);
        state.comments.display_on(w)?;

        // chars hiding
        let after_area = Area::new(area.left + 1, area.bottom() - 3, area.width - 2, 3);
        skin.dialog.md.write_in_area_on(w, MD_AFTER, &after_area)?;

        Ok(())
    }
}


