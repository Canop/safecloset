use {
    super::*,
    crate::tui::*,
    termimad::*,
};

#[derive(Default)]
pub struct FileSelectorView {
    available_area: Area,
}

impl FileSelectorView {
    fn compute_area_width(&self) -> u16 {
        let screen = &self.available_area;
        let sw2 = screen.width / 2;
        let w2 = 27.min(sw2 - 3); // dialog half width
        w2 * 2
    }
    fn compute_area(
        &self,
        content_height: usize,
        area_width: u16,
    ) -> Area {
        let screen = &self.available_area;
        let ideal_height = content_height as u16 + 2; // margin of 1
        let left = (screen.width - area_width) / 2;
        let h = screen.height.min(ideal_height);
        let top = ((screen.height - h) * 3 / 5).max(1);
        Area::new(left, top, area_width, h)
    }
}

impl View<FileSelectorState> for FileSelectorView {
    fn set_available_area(
        &mut self,
        area: Area,
    ) {
        self.available_area = area;
    }

    /// Render the view in its area
    fn draw(
        &mut self,
        w: &mut W,
        state: &mut FileSelectorState, // mutable to allow adapt to terminal size changes
        skin: &AppSkin,
    ) -> Result<(), SafeClosetError> {
        // computing the area from the intro size
        let area_width = self.compute_area_width();
        let text_width = area_width - 2;
        let text = FmtText::from(&skin.dialog.md, &state.intro, Some(text_width as usize));
        let content_height = text.lines.len()
            + 1 // input
            + 1; // 1 for margin
        let area = self.compute_area(content_height, area_width);
        let intro_height = text.lines.len() as u16;

        // border
        let border_colors = skin.dialog.md.table.compound_style.clone();
        let mut rect = Rect::new(area.clone(), border_colors);
        rect.set_fill(true);
        rect.set_border_style(BORDER_STYLE_BLAND);
        rect.draw(w)?;

        // introduction
        let mut area = Area::new(area.left + 1, area.top + 1, text_width, intro_height);
        let mut view = TextView::from(&area, &text);
        view.show_scrollbar = false;
        view.write_on(w)?;

        // file path input
        area.top += intro_height + 1;
        state.input.change_area(area.left, area.top, area.width);
        state.input.display_on(w)?;

        Ok(())
    }
}
